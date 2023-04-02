# Logic Voxels

This is a personal game project created with [Bevy](https://bevyengine.org/).
Development overview and final thoughts at [LogicProjects on Youtube](https://www.youtube.com/watch?v=VoohRYGMjf8).

This is a multiplayer voxel game project using renet networking.  The game can be launched by default in a host mode where a single application will run both the server and the client.  A standalone server can also be launched which will create a gui allowing for the modification of server entities and viewing debugging information about network traffic.  

Clients are able to break and place blocks which will update the chunks on the server which will also send those updates on to other connected clients. The design also features a simple way to send data blocks larger than a single packet size which is used for streaming chunks from the server to the client in a compressed format.  The player moves with WASD, places blocks with right click, and destroys blocks with left click.  The world randomly generates upon the first load but subsequent plays will read the chunks from save files.

The actual chunk meshes are created at run time using a greedy meshing algorithm.  This results in a much lower vertex count allowing for larger worlds to be loaded.

![Example Gif](gifs/greedy_mesh_demo.gif)

The game features integration with rapier physics which allows for the players and physics objects to collide with the dynamically generated chunk mesh.

![Example Gif](gifs/physics_demo.gif)

The organization follows a server client model.  In src/bin there are 3 projects, one for running the game in host mode which is the default, and one each for the server and client.  In the chunks module there is general shared data and functions like blocks and core chunk data types but then within the module there are sub-modules for the specfic client and server side operations (such as greedy mesh creation). Finally the networking module provides a simple wrapper over Renet and turns the raw packets into more friendly Bevy events and other ECS concepts.

All assets and code were created by LogicProjects (mwbryant) and are free to use in any way without restriction.

# Usage

Run the experimental single binary which creates a server and client in 1 app
```
cargo run
```

Run the standalone server
```
cargo run --bin server
```

Run a client
```
cargo run --bin client
```

# Contributions
This project is currently closed to contributions! This is just a personal fun project for me.