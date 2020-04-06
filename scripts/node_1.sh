#!/bin/sh
echo "Running node 1..."
ssh -tt -i ../CSE403_Project.pem ubuntu@ec2-54-144-221-150.compute-1.amazonaws.com << EOF
  cd DistributedHashTable
  cargo run --bin client_application &
  cargo run --bin dht_server &
EOF
