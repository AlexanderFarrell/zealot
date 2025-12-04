docker stack rm zealot &&
docker compose -f zealot-compose.yml build &&
# sleep 3 &&
# docker volume rm zealot_zealot_data &&
docker stack deploy -c zealot-compose.yml zealot
