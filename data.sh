#!/usr/bin/env sh

wget http://mattmahoney.net/dc/enwik8.zip
unzip enwik8.zip
rm enwik8.zip

wget https://g-8d6b0.fd635.8443.data.globus.org/ds131.2/Data-Reduction-Repo/raw-data/EXASKY/NYX/SDRBENCH-EXASKY-NYX-512x512x512.tar.gz
tar -xzvf SDRBENCH-EXASKY-NYX-512x512x512.tar.gz
rm SDRBENCH-EXASKY-NYX-512x512x512.tar.gz
