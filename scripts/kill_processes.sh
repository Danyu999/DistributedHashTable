#!/bin/sh

echo "node 1"
ssh -i ../CSE403_Project.pem ubuntu@ec2-3-80-47-253.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
  exit
EOF

echo "node 2"
ssh -i ../CSE403_Project.pem ubuntu@ec2-204-236-222-8.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
  exit
EOF

echo "node 3"
ssh -i ../CSE403_Project.pem ubuntu@ec2-54-172-169-64.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
  exit
EOF

echo "node 4"
ssh -i ../CSE403_Project.pem ubuntu@ec2-3-80-229-122.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
  exit
EOF

echo "node 5"
ssh -i ../CSE403_Project.pem ubuntu@ec2-54-91-169-21.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
  exit
EOF

killall ssh