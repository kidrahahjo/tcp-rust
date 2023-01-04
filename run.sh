#! /bin/bash

echo "Removing existing interface, if present"
sudo ip link delete tun0

echo "Building"
cargo b --release

echo "Setting capabilities to perform networking operations"
sudo setcap cap_net_admin=eip target/release/tcp-rust

echo "Running the process in the background"
target/release/tcp-rust &
pid=$!

# TODO: There should not be any setup lag
echo "Waiting for the setup of network interface"
sleep 1s

echo "Setting up the network"
sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0

trap "kill $pid" INT TERM
echo "Press Ctrl + C to exit the process ${pid}"
wait $pid
