#!/bin/sh
echo "Running node 2 client..."

cat command_client.sh | ssh -tt -i ../CSE403_Project.pem ubuntu@ec2-54-87-137-56.compute-1.amazonaws.com

#ssh -tt -i ../CSE403_Project.pem ubuntu@ec2-54-87-137-56.compute-1.amazonaws.com << EOF
#  cd DistributedHashTable
#  cargo run --bin dht_server &
#  cargo run --bin client_application &
#EOF