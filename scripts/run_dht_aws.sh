#!/bin/sh
bash run_node_1_server.sh &
bash run_node_2_server.sh &
bash run_node_3_server.sh &
bash run_node_1_client.sh &
bash run_node_2_client.sh &
bash run_node_3_client.sh &