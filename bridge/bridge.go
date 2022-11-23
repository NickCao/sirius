package main

import (
	"bytes"
	"crypto/ed25519"
	"crypto/rand"
	"flag"
	"golang.org/x/crypto/ssh"
	"io"
	"log"
	"net"
	"sync"
)

var addr = flag.String("l", "127.0.0.1:2022", "address to listen on")
var path = flag.String("p", "", "path to socket")

func main() {
	flag.Parse()

	_, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		log.Fatal(err)
	}

	signer, err := ssh.NewSignerFromKey(priv)
	if err != nil {
		log.Fatal(err)
	}

	var config = &ssh.ServerConfig{NoClientAuth: true}
	config.AddHostKey(signer)

	lis, err := net.Listen("tcp", *addr)
	if err != nil {
		log.Fatal(err)
	}

	for {
		conn, err := lis.Accept()
		if err != nil {
			log.Println(err)
			continue
		}
		go func() {
			log.Println(handleConn(conn, config))
		}()
	}
}

func handleConn(conn net.Conn, config *ssh.ServerConfig) error {
	log.Println("new connection")
	sconn, chrs, greqs, err := ssh.NewServerConn(conn, config)
	if err != nil {
		return err
	}
	defer sconn.Close()

	wg := &sync.WaitGroup{}
	go ssh.DiscardRequests(greqs)
	for chr := range chrs {
		wg.Add(1)
		go handleChannelRequest(chr, wg)
	}
	wg.Wait()

	return nil
}

func handleChannelRequest(chr ssh.NewChannel, wg *sync.WaitGroup) error {
	log.Println("new channel")
	defer wg.Done()

	if chr.ChannelType() != "session" {
		return chr.Reject(ssh.UnknownChannelType, "unknown channel type")
	}

	ch, reqs, err := chr.Accept()
	if err != nil {
		return err
	}

	for req := range reqs {
		if req.Type != "exec" {
			if req.WantReply {
				req.Reply(false, nil)
			}
			continue
		}
		if len(req.Payload) < 4 || !bytes.Equal(req.Payload[4:], []byte("nix-daemon --stdio")) {
			if req.WantReply {
				req.Reply(false, nil)
			}
			continue
		}

		uconn, err := net.Dial("unix", *path)
		if req.WantReply {
			req.Reply(err == nil, nil)
		}
		if err != nil {
			return err
		}

		go func() { io.Copy(uconn, ch) }()
		go func() { io.Copy(ch, uconn) }()
	}

	return nil
}
