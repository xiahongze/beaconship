FROM rustembedded/cross:armv7-unknown-linux-gnueabihf-0.2.1

RUN cd /tmp && \
    git clone https://github.com/openssl/openssl && \
    cd openssl && \
    CC=arm-linux-gnueabihf-gcc CXX=arm-linux-gnueabihf-g++ LDLIBS="-latomic" \
    ./config --prefix=/opt/openssl linux-generic32 && \
    make install && \
    cd / && \
    rm -rf /tmp/openssl
