#!/bin/sh

echo "node 1"
ssh -i ../CSE403_Project.pem ubuntu@ec2-54-144-221-150.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
EOF

echo "node 2"
ssh -i ../CSE403_Project.pem ubuntu@ec2-54-146-232-30.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
EOF

echo "node 3"
ssh -i ../CSE403_Project.pem ubuntu@ec2-54-83-178-111.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
EOF

echo "node 4"
ssh -i ../CSE403_Project.pem ubuntu@ec2-54-83-97-2.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
EOF

echo "node 5"
ssh -i ../CSE403_Project.pem ubuntu@ec2-18-208-249-46.compute-1.amazonaws.com << EOF
  killall dht_server
  killall client_application
EOF