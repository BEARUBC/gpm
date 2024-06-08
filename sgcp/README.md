## The Simple Grasp Communication Protocol (SGCP)
A dead-simple protocol built over TCP used to communicate with the primary software module. The exact details of this protocol are still a todo-item and would need more collaboration with component owners. But as an example, imagine we want to send a command to the arm to make a fist, this request might look like:
```
SERVO POS 3
```
which reads as "Set the arm to position 3". Of couse 3 here is an arbitrary number and would be abstracted away by an enum definition on client-side libraries.