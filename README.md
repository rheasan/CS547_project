# CS547 project
---

 - Shreyas Chavan 2001CS67
 - Shubham Ghodke 2001CS68

---
 - Prepare file containing binary data of size k*packet_size

 - Start the sink server 

   ```bash
   $ cd sink
   $ cargo run -- <start_port> <end_port>
   ```
	
 - Start the faucet server

	```bash
	$ cd faucet
	$ cargo run -- <start_port> <end_port> <data_file> <packet_size>
	```