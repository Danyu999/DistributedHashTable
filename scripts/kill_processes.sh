#!/bin/sh

echo "node 1"
ssh -i ../CSE403_Project.pem ubuntu@ec2-54-82-28-5.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
EOF

echo "node 2"
ssh -i ../CSE403_Project.pem ubuntu@ec2-54-242-151-52.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
EOF

echo "node 3"
ssh -i ../CSE403_Project.pem ubuntu@ec2-52-91-53-140.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
EOF