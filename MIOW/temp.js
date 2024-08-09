let miow = miow.new();

// First, we want to create our memory and all that jazz
// This is done by calling the initialize function, and passing an object.
miow.initialize(memory: MiowMemory);

// Now that our memory is good, we want to check our network connection
// This process lasts over a few ticks
// Tick 1 - Get owner segment, see if we can decrypt it. If we can, return true
// Tick 2 - Get the owners terminal, and send the transaction
// Tick 3 - Wait, because the owner is accepting and returning the transaction and returning the key or a cope message
// Tick 4 - Get the response, and if its a key, store it in our MiowMemory object and return true.

// This is done by calling the connect function, and passing an object.
miow.connect(memory: MiowMemory);

// Now that we have our memory and stuff, we want to start reading these segments
miow.read_segments(memory: MiowMemory);
// Say we want to write to our segment, we can do that by calling the write function
miow.write_segment(segment: MiowSegment);

