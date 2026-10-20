Repo to get an offscreen framebuffer backed by OpenGL to render on a terminal supporting Kitty Graphics Protocol.

kp_test: Will be the repo with the above

subcell: Will be the alternate architecture using dma_buf to render with lower overhead

#TODO:

- Metrics
- Right now, kp_test only works on wayland. Make generic to handle x11, too
- Figure out how to do proper cleanup lmao
