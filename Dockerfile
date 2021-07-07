FROM rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.1

RUN apt-get update && \
    apt-get install --assume-yes libsqlite3-dev sqlite3

CMD ["sleep", "infinity"]
