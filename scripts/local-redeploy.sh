docker compose -f build-zealot-compose.yml build &&
docker stack rm zealot &&
echo "Sleeping for Breakdown of Zealot" &&
sleep 30 &&
docker stack deploy -c zealot-compose.yml zealot &&
echo "Deploy Complete" &&
date