#!/bin/bash
###
 # @Author       : BobAnkh
 # @Github       : https://github.com/BobAnkh
 # @Date         : 2021-12-02 16:51:50
 # @LastEditors  : BobAnkh
 # @LastEditTime : 2021-12-15 13:44:36
 # @Description  :
 # Copyright 2021 BobAnkh
###

sleep 5

kill_backend() {
  echo 'Received TERM, gracefully exiting...'
  kill "$(pgrep backend)"
  kill "$(pgrep task_executor)"
  wait $!
  echo 'Process finished'
}

trap 'kill_backend' TERM INT
echo "[ENTRYPOINT] Running Background Task Executor"
task_executor &
echo "[ENTRYPOINT] Running Backend"
backend &
wait $!
