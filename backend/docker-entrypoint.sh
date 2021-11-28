#!/bin/bash

echo "Running consumer"
consumer &
echo "Running backend"
backend
