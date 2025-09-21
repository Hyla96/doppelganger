package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/http"
	"time"
)

const (
	DefaultPort  = 3003
	DelaySeconds = 1
)

type Response struct {
	Message   string `json:"message"`
	Delay     string `json:"delay"`
	Timestamp string `json:"timestamp"`
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write([]byte(`{"status":"healthy"}`))
}

func exampleHandler(w http.ResponseWriter, r *http.Request) {
	fmt.Printf("Called exanple endpoint")

	time.Sleep(DelaySeconds * time.Second)

	response := Response{
		Message:   "Go service response",
		Delay:     fmt.Sprintf("%d second", DelaySeconds),
		Timestamp: time.Now().UTC().Format(time.RFC3339),
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)

	if err := json.NewEncoder(w).Encode(response); err != nil {
		http.Error(w, "Failed to encode response", http.StatusInternalServerError)
		return
	}
}

func main() {
	port := flag.Int("port", DefaultPort, "port to listen on")
	flag.Parse()

	http.HandleFunc("/health", healthHandler)
	http.HandleFunc("/example-endpoint", exampleHandler)

	addr := fmt.Sprintf(":%d", *port)
	fmt.Printf("V2 service listening on http://0.0.0.0%s\n", addr)

	if err := http.ListenAndServe(addr, nil); err != nil {
		log.Fatal("Server failed to start:", err)
	}
}

