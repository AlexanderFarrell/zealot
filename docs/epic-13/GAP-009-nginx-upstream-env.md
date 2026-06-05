# GAP-009 — Make nginx upstream configurable (not hardcoded)

## Problem

`apps/web/nginx.conf` hardcodes the upstream as:

```
proxy_pass http://zealot_zealot:8456/;
```

`zealot_zealot` is Docker Compose's internal DNS name for a service named `zealot`
in a project named `zealot`. If either the project name or the service name changes,
the web image silently breaks — every API request returns a 502, because nginx cannot
resolve the upstream hostname.

This is documented as a known limitation:

> "The nginx upstream is hardcoded. `apps/web/nginx.conf` references
> `zealot_zealot:8456`. If you change the Compose project name or service name, the
> web image must be rebuilt with an updated `nginx.conf`."

The limitation surfaces when:
- An operator runs with `-p my-project` or sets `COMPOSE_PROJECT_NAME`
- Someone forks the repo and changes the compose service name
- A Docker Swarm or Kubernetes deployment uses a different service name

## Proposed fix

Use an environment variable in `nginx.conf` with `envsubst`:

1. Change `nginx.conf` to use a variable for the upstream:
   ```nginx
   resolver 127.0.0.11 valid=5s;
   set $upstream ${ZEALOT_UPSTREAM:-zealot_zealot:8456};
   proxy_pass http://$upstream/;
   ```

2. In the `web` Dockerfile's CMD, run `envsubst` on the template before starting nginx:
   ```dockerfile
   CMD ["/bin/sh", "-c", "envsubst '${ZEALOT_UPSTREAM}' < /etc/nginx/nginx.conf.template > /etc/nginx/conf.d/default.conf && nginx -g 'daemon off;'"]
   ```

3. The `ZEALOT_UPSTREAM` env var defaults to `zealot_zealot:8456` in the compose files
   so existing deployments require no change.

## Files to change

- `apps/web/nginx.conf` → rename to `nginx.conf.template`, use `$ZEALOT_UPSTREAM`
- `apps/web/Dockerfile` — add `envsubst` step
- `scripts/zealot-compose.yml` and `docker-compose.yml` — optionally set `ZEALOT_UPSTREAM`
- `docs/deployment.md` — remove known limitation, document the variable
- `docs/troubleshooting.md` — update nginx upstream section
