#!/bin/bash

while [[ "$#" -gt 0 ]]; do                                                                                     
    case $1 in
        --release) release=true ;;
        *) echo "Unknown parameter passed: $1"; exit 1 ;;
    esac
    shift
done

if [ "$release" = true ]; then
    cargo build --release --target aarch64-unknown-linux-gnu                                                              
    if [ $? -eq 0 ]; then                                                       
        rsync -avz --delete /home/joel/GH/Lights/light-crud-api/target/aarch64-unknown-linux-gnu/release/light-crud-api pi@192.168.2.39:/home/pi/light-crud-api                                 
    fi 
else
    echo "Warning: This seems to not work on the raspberry pi, use --release"
    cargo build --target aarch64-unknown-linux-gnu                                                              
    if [ $? -eq 0 ]; then                                                       
        rsync -avz --delete /home/joel/GH/Lights/light-crud-api/target/aarch64-unknown-linux-gnu/debug/light-crud-api pi@192.168.2.39:/home/pi/light-crud-api                                 
    fi 
fi

# cargo build --release --target aarch64-unknown-linux-gnu                                                              
# if [ $? -eq 0 ]; then                                                       
#    rsync -avz --delete /home/joel/GH/Lights/light-crud-api/target/aarch64-unknown-linux-gnu/release/light-crud-api pi@192.168.2.39:/home/pi/light-crud-api                                 
# fi                                                                          
