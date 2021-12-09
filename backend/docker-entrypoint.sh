#!/bin/bash
###
 # @Author       : BobAnkh
 # @Github       : https://github.com/BobAnkh
 # @Date         : 2021-12-02 16:51:50
 # @LastEditors  : BobAnkh
 # @LastEditTime : 2021-12-08 08:09:45
 # @Description  :
 # Copyright 2021 BobAnkh
###

echo "Running Background Task"
task_executor &
echo "Running Backend"
backend
