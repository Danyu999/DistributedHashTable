#!/bin/sh

echo "node 1"
cat kill_commands.sh | ssh -i ../CSE403_Project.pem ubuntu@ec2-54-160-219-251.compute-1.amazonaws.com

echo "node 2"
cat kill_commands.sh | ssh -i ../CSE403_Project.pem ubuntu@ec2-54-87-137-56.compute-1.amazonaws.com

echo "node 3"
cat kill_commands.sh | ssh -i ../CSE403_Project.pem ubuntu@ec2-54-227-82-9.compute-1.amazonaws.com