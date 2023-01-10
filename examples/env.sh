#!/bin/bash

DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

export VK_ADD_LAYER_PATH="$DIR"
export VK_LOADER_LAYERS_ENABLE="VK_LAYER_ASH_LAYER_dummy"
