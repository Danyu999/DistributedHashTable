#!/bin/sh
echo "Running node 5..."
ssh -tt -i ../CSE403_Project.pem ubuntu@ec2-52-91-53-140.compute-1.amazonaws.com << EOF
  cd DistributedHashTable
  cargo run --bin client_application &
  cargo run --bin dht_server &
EOF