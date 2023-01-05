#! /bin/bash

# If compilation fails, do not run anything

echo "Removing existing interface, if present"
sudo ip link delete tun0

cargo b --release
# If build fails, exit early
ext=$?
if [[ $ext -ne 0 ]] 
then
    exit $ext
fi

echo "Setting capabilities to perform networking operations"
sudo setcap cap_net_admin=eip target/release/tcp-rust

echo "Running the process in the background"
target/release/tcp-rust &
pid=$!

echo "Setting up the network"
sleep 1
sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0

trap "kill $pid" INT TERM
echo "Press Ctrl + C to exit the process ${pid}"
wait $pid
