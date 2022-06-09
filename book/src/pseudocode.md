# Draft
cb: I am not using the correct syntax here yet,
I am just writing the text and listing the items
which I'd like to include.
TODO:
- discuss brackets around statements


# Game Definitions
## Types
The type of a variable specifies which
kind of values it can take, e.g., whether
x is an integer (Int) or a bistring (bits*).
While pseudocode in the cryptographic literature
usually does not specify the type of variables
explicitly, types are useful for formal verification,
so we require pseudo-code to specify the type of each 
variable, e.g., x: Bits(*).

### Supported Types
Our pseudo-code supports the following types:
- TODO
- TODO
- TODO

### Bot/None and Maybe Types
In pseudo-code, we often want to check whether a
variable has already been assigned or whether a table
has already been assigned at a particular index. In
pseudocode, we encode this by checking whether
x = none(Bits(*)). Here, a subtlety in typing comes
into place. Namely, if x is supposed to be of type
Bits(*), then the value "unassigned" is not of type
Bits(*). Thus, if there is a possibility for x to be
unassigned, then x needs to be originally declared
as being of Maybe type which means, either bits*
or none(Bits(*)). We write x : Maybe(Bits(*)). Indeed,
since in security games, most variables can be
undefined, most variables will indeed be of type
Maybe(type). Given any type type, the constructor

Maybe(type)

builds a new type which, in addition to type, includes
a none symbol. If x is of type type, then we can use
the operator


to obtain a variable which has the same value as x,
but is of type Maybe.

#### tuple types
Given n types type_1,..,type_n, the following tuple
type is also defined:

TODO

The tuple operator for constructing types can be
applied recursively, i.e., type_1,...,type_n might
also already be tuple types.


#### table types
Given two types type_1 and type_2, we define
a table with entries of type_2 and indices of type_1
as follows:

TODO

#### function types

TODO


### Type Errors
You will get a typecheck error when...


## Pseudocode
Our pseudocode style follows popular convention used in
the cryptographic literature.

### Statements

Assignments
x <- expr

Table Assignments
T[x] <- y

Sampling
x <-$ S

### Expressions
TODO


## Packages
A package is a base objects, of which multiple *instances*
can later be used, cf. Section~\ref{X}. It consists of 
a *state*, *parameters*, the set of *oracles* it defines
as well as the set of oracles it calls (imports).

### State
### Parameters
### Oracles
### Oracle imports


## Package Instances
Each package instance needs to specify the *name* of the
package instance, the *parameters* which this package
instance should use as well as the *types* of the
arguments which it takes as well as the types of
the tables, arguments, and return values.

### Package Instance name
### Parameters


## Composition