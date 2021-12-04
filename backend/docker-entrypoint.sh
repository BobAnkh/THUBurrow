#!/bin/bash
###
 # @Author       : BobAnkh
 # @Github       : https://github.com/BobAnkh
 # @Date         : 2021-12-02 16:51:50
 # @LastEditors  : BobAnkh
 # @LastEditTime : 2021-12-04 16:57:49
 # @Description  :
 # Copyright 2021 BobAnkh
###

echo "Running consumer"
consumer &
echo "Running backend"
backend
