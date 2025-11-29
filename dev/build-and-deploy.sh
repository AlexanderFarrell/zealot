docker stack rm zealot &&
docker compose -f zealot-compose.yml build &&
docker stack deploy -c zealot-compose.yml zealot
