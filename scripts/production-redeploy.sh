docker compose -f build-zealot-compose.yml build &&
docker compose -f build-zealot-compose.yml push  &&
ssh citadel "cd /home/alexander/projects/citadel/docker && docker pull registry.alexanderfarrell.net/zealotd:v0.1.0 && docker pull registry.alexanderfarrell.net/zealot-web:v0.1.0 && docker stack rm core && docker stack rm zealot && sleep 10 && docker stack deploy -c core-compose.yml core && sleep 5 && docker stack deploy -c zealot-compose.yml zealot"
