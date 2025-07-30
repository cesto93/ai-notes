D-Bus is a system for low-overhead, easy to use interprocess communication (IPC). In more detail:
- D-Bus is _low-overhead_ because it uses a binary protocol, and does not have to convert to and from a text format such as XML.  D-Bus is also designed to avoid round trips and allow asynchronous operation, much like the X protocol.
- D-Bus is _easy to use_ because it works in terms of _messages_ rather than byte streams. The D-Bus library is designed to be wrapped in a way that lets developers use their framework's existing object/type system, rather than learning a new one specifically for IPC.

The base D-Bus protocol is a one-to-one (peer-to-peer or client-server) protocol, specified in [the section called “Message Protocol”](https://dbus.freedesktop.org/doc/dbus-specification.html#message-protocol "Message Protocol"). That is, it is a system for one application to talk to a single other application. However, the primary intended application of the protocol is the D-Bus _message bus_, specified in [the section called “Message Bus Specification”](https://dbus.freedesktop.org/doc/dbus-specification.html#message-bus "Message Bus Specification"). The message bus is a special application that accepts connections from multiple other applications, and forwards messages among them.

Uses of D-Bus include notification of system changes (notification of when a camera is plugged in to a computer, or a new version of some software has been installed), or desktop interoperability, for example a file monitoring service or a configuration service.

D-Bus is designed for two specific use cases:
- A "system bus" for notifications from the system to user sessions, and to allow the system to request input from user sessions.
- A "session bus" used to implement desktop environments such as GNOME and KDE.

D-Bus is not intended to be a generic IPC system for any possible application, and intentionally omits many features found in other IPC systems for this reason.

### Valid Object Paths
An object path is a name used to refer to an object instance. Conceptually, each participant in a D-Bus message exchange may have any number of object instances (think of C++ or Java objects) and each such instance will have a path. Like a filesystem, the object instances in an application form a hierarchical tree.
Object paths are often namespaced by starting with a reversed domain name and containing an interface version number.
This makes it possible to implement more than one service, or more than one version of a service, in the same process.



##### Bibliography
https://dbus.freedesktop.org/doc/dbus-specification.html#message-bus-routing-match-rules