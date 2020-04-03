#!/bin/sh
echo "Running node 3 client..."

cat command_client.sh | ssh -tt -i ../CSE403_Project.pem ubuntu@ec2-54-227-82-9.compute-1.amazonaws.com

#ssh -tt -i ../CSE403_Project.pem ubuntu@ec2-54-227-82-9.compute-1.amazonaws.com << EOF
#  cd DistributedHashTable
#  cargo run --bin dht_server &
#  cargo run --bin client_application &
#EOF