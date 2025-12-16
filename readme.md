# Zealot

![Zealot Logo](./client/public/zealot.webp)

Powerful planner, wiki and Life OS tool. 

# Get Started

## Docker

To build all containers, run the following:

```sh
cd dev &&
docker compose -f zealot-compose.yml build
```


# Vetting

To scan the code for security issues, install the go vetting tool:

```
go install honnef.co/go/tools/cmd/staticcheck@latest
./zdev vet
```