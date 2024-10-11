FROM ubuntu:latest
ADD https://github.com/lightron/lightron/releases/download/v0.1.0/lightron-0.1.0.deb /root/lightron-0.1.0.deb
RUN apt-get update && apt-get install -y /root/lightron-0.1.0.deb
EXPOSE 80/tcp
ENTRYPOINT /usr/bin/lightron-core