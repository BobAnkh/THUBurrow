#!/bin/bash
###
# @Author       : BobAnkh
# @Github       : https://github.com/BobAnkh
# @Date         : 2021-12-02 16:51:50
 # @LastEditors  : BobAnkh
 # @LastEditTime : 2021-12-29 13:50:40
# @Description  :
# Copyright 2021 BobAnkh
###

kill_backend() {
    echo "$(date -u "+%Y-%m-%d %H:%M:%S UTC") Received TERM, gracefully exiting..."
    kill "$(pgrep backend)"
    kill "$(pgrep task-executor)"
    wait "$(pgrep task-executor)"
    echo "$(date -u "+%Y-%m-%d %H:%M:%S UTC") Process finished"
}

trap 'kill_backend' TERM INT
echo "$(date -u "+%Y-%m-%d %H:%M:%S UTC") [ENTRYPOINT] Running Background Task Executor"
task-executor &
echo "$(date -u "+%Y-%m-%d %H:%M:%S UTC") [ENTRYPOINT] Running Backend"
backend &
wait $!
