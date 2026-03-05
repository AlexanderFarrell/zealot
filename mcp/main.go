package main

import (
	"context"
	"errors"
	"flag"
	"log"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/mark3labs/mcp-go/server"
)

const (
	defaultZealotAPIURL = "https://zealot.alexanderfarrell.net"
	defaultTransport    = "http"
	defaultListenAddr   = ":8080"
	defaultBasePath     = "/mcp"
	defaultSSEEndpoint  = "/sse"
	defaultMsgEndpoint  = "/message"
)

type config struct {
	apiURL          string
	apiToken        string
	transport       string
	listenAddr      string
	basePath        string
	sseEndpoint     string
	messageEndpoint string
}

func getenvOrDefault(key, fallback string) string {
	value := strings.TrimSpace(os.Getenv(key))
	if value == "" {
		return fallback
	}
	return value
}

func normalizePath(raw, fallback string) string {
	path := strings.TrimSpace(raw)
	if path == "" {
		path = fallback
	}
	if !strings.HasPrefix(path, "/") {
		path = "/" + path
	}
	return path
}

func normalizeBasePath(raw, fallback string) string {
	base := strings.TrimSpace(raw)
	if base == "" {
		base = fallback
	}
	if base == "/" {
		return ""
	}
	if !strings.HasPrefix(base, "/") {
		base = "/" + base
	}
	return strings.TrimSuffix(base, "/")
}

func endpointPath(basePath, endpoint string) string {
	return strings.TrimSuffix(basePath, "/") + endpoint
}

func readConfig() config {
	cfg := config{
		apiURL:          getenvOrDefault("ZEALOT_API_URL", defaultZealotAPIURL),
		apiToken:        strings.TrimSpace(os.Getenv("ZEALOT_API_TOKEN")),
		transport:       strings.ToLower(getenvOrDefault("MCP_TRANSPORT", defaultTransport)),
		listenAddr:      getenvOrDefault("MCP_LISTEN_ADDR", defaultListenAddr),
		basePath:        getenvOrDefault("MCP_BASE_PATH", defaultBasePath),
		sseEndpoint:     getenvOrDefault("MCP_SSE_ENDPOINT", defaultSSEEndpoint),
		messageEndpoint: getenvOrDefault("MCP_MESSAGE_ENDPOINT", defaultMsgEndpoint),
	}

	flag.StringVar(&cfg.apiURL, "api-url", cfg.apiURL, "Zealot API base URL")
	flag.StringVar(&cfg.apiToken, "api-token", cfg.apiToken, "Zealot API token")
	flag.StringVar(&cfg.transport, "transport", cfg.transport, "MCP transport: http or stdio")
	flag.StringVar(&cfg.listenAddr, "listen", cfg.listenAddr, "HTTP listen address (http transport)")
	flag.StringVar(&cfg.basePath, "base-path", cfg.basePath, "HTTP base path for MCP endpoints")
	flag.StringVar(&cfg.sseEndpoint, "sse-endpoint", cfg.sseEndpoint, "SSE endpoint path")
	flag.StringVar(&cfg.messageEndpoint, "message-endpoint", cfg.messageEndpoint, "Message endpoint path")
	flag.Parse()

	cfg.apiURL = strings.TrimSpace(cfg.apiURL)
	cfg.apiToken = strings.TrimSpace(cfg.apiToken)
	cfg.transport = strings.ToLower(strings.TrimSpace(cfg.transport))
	cfg.basePath = normalizeBasePath(cfg.basePath, defaultBasePath)
	cfg.sseEndpoint = normalizePath(cfg.sseEndpoint, defaultSSEEndpoint)
	cfg.messageEndpoint = normalizePath(cfg.messageEndpoint, defaultMsgEndpoint)

	return cfg
}

func buildMCPServer(client *ZealotClient) *server.MCPServer {
	s := server.NewMCPServer(
		"zealot-mcp",
		"1.0.0",
		server.WithToolCapabilities(false),
	)
	registerTools(s, client)
	return s
}

func runStdio(mcpServer *server.MCPServer) error {
	return server.NewStdioServer(mcpServer).Listen(context.Background(), os.Stdin, os.Stdout)
}

func runHTTP(mcpServer *server.MCPServer, cfg config) error {
	sseOptions := []server.SSEOption{
		server.WithBasePath(cfg.basePath),
		server.WithSSEEndpoint(cfg.sseEndpoint),
		server.WithMessageEndpoint(cfg.messageEndpoint),
	}

	httpServer := &http.Server{
		Addr: cfg.listenAddr,
	}
	sseOptions = append(sseOptions, server.WithHTTPServer(httpServer))
	sseServer := server.NewSSEServer(mcpServer, sseOptions...)

	mux := http.NewServeMux()
	mux.Handle(endpointPath(cfg.basePath, cfg.sseEndpoint), sseServer)
	mux.Handle(endpointPath(cfg.basePath, cfg.messageEndpoint), sseServer)
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})
	httpServer.Handler = mux

	log.Printf("MCP HTTP server listening on %s", cfg.listenAddr)
	log.Printf("MCP SSE endpoint: %s", endpointPath(cfg.basePath, cfg.sseEndpoint))
	log.Printf("MCP message endpoint: %s", endpointPath(cfg.basePath, cfg.messageEndpoint))

	serverErr := make(chan error, 1)
	go func() {
		serverErr <- httpServer.ListenAndServe()
	}()

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	select {
	case err := <-serverErr:
		if err != nil && !errors.Is(err, http.ErrServerClosed) {
			return err
		}
		return nil
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := sseServer.Shutdown(shutdownCtx); err != nil && !errors.Is(err, http.ErrServerClosed) {
			return err
		}
		return nil
	}
}

func main() {
	cfg := readConfig()

	if cfg.apiURL == "" {
		cfg.apiURL = defaultZealotAPIURL
	}
	if cfg.apiToken == "" {
		log.Fatal("ZEALOT_API_TOKEN environment variable is required")
	}

	client := NewZealotClient(cfg.apiURL, cfg.apiToken)
	mcpServer := buildMCPServer(client)

	var err error
	switch cfg.transport {
	case "", "http", "sse":
		err = runHTTP(mcpServer, cfg)
	case "stdio":
		err = runStdio(mcpServer)
	default:
		log.Fatalf("unsupported MCP_TRANSPORT %q (expected http or stdio)", cfg.transport)
	}

	if err != nil {
		log.Fatalf("MCP server error: %v", err)
	}
}
