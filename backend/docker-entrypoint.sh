#!/bin/bash
###
 # @Author       : BobAnkh
 # @Github       : https://github.com/BobAnkh
 # @Date         : 2021-12-02 16:51:50
 # @LastEditors  : BobAnkh
 # @LastEditTime : 2021-12-10 06:29:52
 # @Description  :
 # Copyright 2021 BobAnkh
###

sleep 5
echo "Running Background Task"
task_executor &
echo "Running Backend"
backend
