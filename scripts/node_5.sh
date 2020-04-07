#!/bin/sh
echo "Running node 5..."
ssh -tt -i ../CSE403_Project.pem ubuntu@ec2-3-91-6-246.compute-1.amazonaws.com << EOF
  cd DistributedHashTable
  cargo run --bin dht_server &
  cargo run --bin client_application
  sleep 10s
  killall dht_server
  exit
EOF