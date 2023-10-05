FROM ubuntu:22.04

RUN apt update -y && apt install -y mosquitto

WORKDIR /ws

COPY mosquitto.conf .

ENTRYPOINT ["mosquitto", "-c", "mosquitto.conf"]
