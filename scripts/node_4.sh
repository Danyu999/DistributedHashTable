#!/bin/sh
echo "Running node 4..."
ssh -tt -i ../CSE403_Project.pem ubuntu@ec2-18-207-121-165.compute-1.amazonaws.com << EOF
  cd DistributedHashTable
  cargo run --bin dht_server &
  cargo run --bin client_application
  sleep 10s
  killall dht_server
  exit
EOF