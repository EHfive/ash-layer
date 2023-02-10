## Examples

1. Build the dummy layer
```bash
cargo build --examples
```

2. Add layer manifest to Vulkan loader lookup path and enable the layer
```bash
# export VK_ADD_LAYER_PATH=$(pwd)
# export VK_LOADER_LAYERS_ENABLE="VK_LAYER_ASH_LAYER_dummy"
source .envrc
```

3. Run any Vulkan APP
```bash
vkcube
```
