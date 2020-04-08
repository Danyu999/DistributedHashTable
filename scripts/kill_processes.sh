#!/bin/sh

echo "node 1"
ssh -i ../CSE403_Project.pem ubuntu@ec2-34-227-191-28.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
  exit
EOF

echo "node 2"
ssh -i ../CSE403_Project.pem ubuntu@ec2-18-212-252-181.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
  exit
EOF

echo "node 3"
ssh -i ../CSE403_Project.pem ubuntu@ec2-3-88-99-222.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
  exit
EOF

echo "node 4"
ssh -i ../CSE403_Project.pem ubuntu@ec2-18-207-121-165.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
  exit
EOF

echo "node 5"
ssh -i ../CSE403_Project.pem ubuntu@ec2-3-91-6-246.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
  exit
EOF

killall ssh