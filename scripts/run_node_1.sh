#!/bin/sh
ssh -tt -i ../CSE403_Project.pem ubuntu@AWSURL

cargo run --bin dht_server &
cargo run --bin client_application &