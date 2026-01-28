# High-Quality Haskell Codebases

A curated list of production-grade Haskell projects to study for patterns, idioms, and best practices.

---

## By Maintainer/Contributor

### Simon Marlow & GHC Team
- **GHC** (The Glasgow Haskell Compiler)
  - Purpose: Production compiler for Haskell
  - Repo: https://gitlab.haskell.org/ghc/ghc
  - Size: ~600K LOC, multi-year evolution
  - Key patterns: Compiler architecture, monadic codegen, abstract interpretation

### Edward Kmett (ekmett)
- **lens** - https://github.com/ekmett/lens
  - Purpose: Modern optics library for Haskell
  - Patterns: Type families, generic programming, category theory encoding

- **pipes** - https://github.com/Gabriel439/haskell-pipes
  - Purpose: Composable streaming (contributor)
  - Patterns: Resource-safe effect systems, proxy patterns

- **free** - https://github.com/ekmett/free
  - Purpose: Free monads and their comonads
  - Patterns: Higher-kinded data, recursion schemes

- **adjunctions** - https://github.com/ekmett/adjunctions
  - Purpose: Adjunctions and representable functors
  - Patterns: Category theory abstractions

- **kan-extensions** - https://github.com/ekmett/kan-extensions
  - Purpose: Kan extensions
  - Patterns: Advanced category theory in Haskell

### Oleg Kiselyov
- **Tagless Final** (papers + examples)
  - Purpose: Compile-time metaprogramming without Template Haskell
  - Patterns: Type classes as language DSLs, final encoding

- **typed-tagless-transform**
  - Repo: https://github.com/namin/typed-tagless-transform
  - Patterns: Staged interpretation, polymorphism

- **LogicT** (monad transformer for backtracking)
  - Patterns: Effect composition, control flow

### Philip Wadler
- **Parsec** (original contributions)
  - Purpose: Monadic parser combinator library
  - Patterns: Combinator composition, error reporting

- **Wadler's monads papers (foundational)**

### Ben Gamari & Well-Typed
- **GHC** (ongoing maintenance)
- See GHC patterns below

### Gabriel Gonzalez
- **pipes** - https://github.com/Gabriel439/haskell-pipes
  - Purpose: Coroutine-based streaming
  - Patterns: Resource management, effect categories

- **turtle** - https://github.com/Gabriel439/turtle
  - Purpose: Shell scripting in Haskell
  - Patterns: DSL design, effect systems

- **optparse-applicative** - https://github.com/pcapriotti/optparse-applicative
  - Purpose: Command-line parsing
  - Patterns: Applicative parsing, builder pattern

---

## By Function/Domain

### Compilers & Language Infrastructure

#### **GHC** (The Glasgow Haskell Compiler)
- **Repo**: https://gitlab.haskell.org/ghc/ghc
- **Purpose**: Production Haskell compiler
- **Scale**: ~600K LOC, 20+ years of evolution

**Key Patterns:**
- **Monadic code generation**: `GHC.Types.Id`, `GHC.Core.Opt.Monad`
- **Abstract interpretation**: For optimization passes
- **Plugin system**: Loadable compiler passes
- **Multi-stage compilation**: HIR → Core → STG → Cmm → Asm
- **Info tables**: Runtime type representation
- **Constraint solving**: Type class resolution
- **Primop encoding**: Low-level operations with safe interface
- **Name resolution**: Uniqueness and renaming
- **Source location tracking**: Span-ful AST

**Study order:**
1. `GHC.Types.Name` - Name representation and uniqueness
2. `GHC.Core` - Core language (STG, simplification)
3. `GHC.Tc` - Type checker and constraint solver
4. `GHC.Driver` - Pipeline orchestration

#### **GHC Core IR**
- **STG (Spineless Tagless G-machine)**: Lazy evaluation
- **Cmm**: Low-level imperative IR
- **Pattern**: Multi-pass optimization pipeline

### Data Processing & Analytics

#### **Glean** (Meta/Facebook)
- **Repo**: https://github.com/facebookincubator/gleam
  (Note: Check official Meta repos - may be private; public documentation available)
- **Purpose**: Fact storage and query system for code analysis
- **Scale**: Petabytes of metadata at Meta

**Key Patterns:**
- **Datalog-based query language**: Declarative fact querying
- **Schema-driven**: Strong typing for facts
- **Incremental indexing**: Efficient updates
- **RPC integration**: FFI for language tools
- **Fact graph**: Representing code relationships

### Parsing & DSL Construction

#### **Parsec** (original by Daan Leijen, influenced by Wadler)
- **Repo**: https://github.com/haskell/parsec
- **Purpose**: Monadic parser combinators
- **Patterns**:
  - **Applicative parsing**: `<?>` operator for error context
  - **Backtracking**: Explicit `try` combinator
  - **User state**: `StateT` integration

#### **Megaparsec** (modern Parsec evolution)
- **Repo**: https://github.com/mrkkrp/megaparsec
- **Purpose**: Industrial-strength parsing
- **Patterns**:
  - **Error reporting**: Rich error types with position
  - **Token-level vs character-level**: Composable layers
  - **Custom error messages**: `Display`-based errors

#### **Alex & Happy** (lex/yacc for Haskell)
- **Purpose**: Lexical analyzer and LALR parser generator
- **Patterns**: Generated tables, monadic actions

### Optics & Data Access

#### **lens** (Edward Kmett)
- **Repo**: https://github.com/ekmett/lens
- **Purpose**: Modern optics library
- **Patterns**:
  - **Van Laarhoven encoding**: `(a -> f b) -> s -> f t`
  - **Type families**: Overloaded labels (`_1`, `_2`, `name`)
  - **Generics**: `makeLenses`, `makePrisms` TH
  - **Category of optics**: Composition, laws

#### **generic-lens** (modern approach)
- **Repo**: https://github.com/kcsongor/generic-lens
- **Purpose**: Derive lenses without Template Haskell
- **Patterns**: Generics, type-level programming

### Streaming & Effect Systems

#### **pipes** (Gabriel Gonzalez)
- **Repo**: https://github.com/Gabriel439/haskell-pipes
- **Purpose**: Coroutine-based streaming
- **Patterns**:
  - **Proxy types**: Request/response pattern
  - **Resource safety**: Finalizers via bracket pattern
  - **Composition**: Category of pipes

#### **conduit** (Michael Snoyman)
- **Repo**: https://github.com/snoyberg/conduit
- **Purpose**: Streaming with early termination
- **Patterns**:
  - **Source/Conduit/Sink**: Flow types
  - **Resource management**: Built-in finalization

#### **io-streams** (Simon Meier)
- **Repo**: https://github.com/simonmeier/io-streams
- **Purpose**: Fast IO with resource safety
- **Patterns**: Handle abstraction, buffer management

### Type-Level Programming

#### **singletons** (Richard Eisenberg)
- **Repo**: https://github.com/goldfirere/singletons
- **Purpose**: Dependently-typed Haskell
- **Patterns**:
  - **Singleton types**: Runtime representatives of types
  - **Promotion**: Term-level → type-level
  - **Template Haskell**: Code generation

#### **kind-generics** (Katherine Coombes)
- **Purpose**: Generic programming at kind level
- **Patterns**: Generic representation, type-level recursion

### Testing & Property-Based Testing

#### **QuickCheck** (Koen Claessen, John Hughes)
- **Repo**: https://github.com/nick8325/quickcheck
- **Purpose**: Randomized property testing
- **Patterns**:
  - **Arbitrary**: Random generation
  - **Shrinking**: Counterexample reduction
  - **Properties**: First-class test values

#### **hedgedhog** (Nick Partridge)
- **Repo**: https://github.com/hedgehogqa/haskell-hedgehog
- **Purpose**: Integrated shrinking
- **Patterns**:
  - **State machine testing**: Sequenced actions
  - **Property combinators**: Integration with HSpec

### Database & Persistence

#### **Persistent** (Michael Snoyman)
- **Repo**: https://github.com/yesodweb/persistent
- **Purpose**: Type-safe SQL DSL
- **Patterns**:
  - **Template Haskell**: Schema → types
  - **Migration**: Automated schema changes
  - **Quasi-quoters**: Raw SQL with interpolation

#### **Beam** (Travis Athougies)
- **Repo**: https://github.com/tathougies/beam
- **Purpose**: Relationally complete SQL
- **Patterns**:
  - **Lenses for rows**: Type-safe column access
  - **Profunctor encoding**: Query composition

### Web Development

#### **Servant** (Alp Mestanogullari, etc.)
- **Repo**: https://github.com/haskell-servant/servant
- **Purpose**: Type-safe web API
- **Patterns**:
  - **Type-level routes**: API as type
  - **Combinator library**: `/ :> Capture, Get, Post`
  - **Deriving content**: Generic serialization

#### **Spock** (Alexander Thiemann)
- **Repo**: https://github.com/agrafix/Spock
- **Purpose**: Sinatra-like routing
- **Patterns**:
  - **Monad transformer stack**: `SpockM`, `ActionM`
  - **Route matching**: Combinator-based

#### **Scotty** (Oscar Boykin)
- **Repo**: https://github.com/scotty-web/scotty
- **Purpose**: Minimalist WAI router
- **Patterns**:
  - **WAI integration**: Web Application Interface
  - **Monad transformer**: `ScottyM`, `ActionM`

### Concurrency & Parallelism

#### **async** (Simon Marlow)
- **Repo**: https://github.com/simonmar/async
- **Purpose**: Async operations with structured concurrency
- **Patterns**:
  - **Async monad**: Composable async actions
  - **Resource management**: `withAsync`, `wait`

#### **stm** (in `base`, Simon Marlow)
- **Purpose**: Software transactional memory
- **Patterns**:
  - **Transaction monad**: `STM`
  - **Composable operations**: `retry`, `orElse`
  - **TVars**: Transactional variables

#### **unagi-chan** (Merijn Verstraaten)
- **Repo**: https://github.com/jberryman/unagi-chan
- **Purpose**: Unbounded queues with blocking operations
- **Patterns**:
  - **SPSC channels**: Single-producer, single-consumer
  - **STM-based**: Retry semantics

---

## Study Recommendations

### For Recursion Schemes
- **lens**: See `Control.Lens.Fold` for cata-like patterns
- **free**: Free monads for interpretable DSLs
- **adjunctions**: Comonad-algebra patterns

### For Compiler Architecture
- **GHC**: The reference implementation
  - Study: `GHC.Core`, `GHC.Tc.Solver`, `GHC.Driver.Pipeline`
- **Glean**: See fact indexing and query optimization

### For DSL Design
- **Parsec/Megaparsec**: Parser combinators
- **Servant**: Type-level routing
- **optparse-applicative**: CLI DSL

### For Effect Systems
- **pipes/conduit**: Streaming
- **mtl**: Monad transformers (the de facto standard)
- **polysemy**: Modern effect system

---

## Pattern Categories Found in These Codebases

### 1. Type-Level Patterns
- Type families for generic programming
- GADTs for invariants
- DataKinds for type-level data
- Quantified constraints for power typeclasses
- Type applications for explicit type arguments

### 2. Effect & Resource Management
- Monad transformers (mtl pattern)
- Bracket pattern for cleanup
- Resource-safe composable IO
- STM for composable concurrency
- Async patterns for structured concurrency

### 3. Generic Programming
- Generics (GHC.Generics)
- Template Haskell code generation
- Scrap-your-boilerplate (SYB)
- Generic-lens for deriving optics

### 4. Combinator Libraries
- Parser combinators (Parsec, Megaparsec)
- Optics combinators (lens)
- Test property combinators (QuickCheck, hedgehog)
- Routing combinators (Servant, Scotty)

### 5. Compiler & IR Patterns
- Multi-stage pipelines
- Source location tracking
- Name resolution environments
- Constraint solving
- Optimization passes as monads

### 6. Streaming & Incremental Processing
- Producer/consumer patterns
- Backpressure handling
- Resource finalization
- Chunked processing

---

## Contributing Patterns

### Open Source Maintenance
- **Haskell CI**: Standardized testing pipeline
- **Stack/Cabal**: Build systems
- **Travis/GitHub Actions**: CI workflows
- **Stackage**: Stable package sets

### Code Organization
- **Cabal files**: Package metadata
- **Exposed modules**: Public API
- **Internal modules**: Implementation hiding
- **Setup.hs**: Custom build hooks

---

## Notes

- Many Meta/Facebook projects (Glean, Hack) are private or have limited public access
- GHC is the largest and most complex Haskell codebase
- Edward Kmett's libraries showcase advanced category theory applied practically
- Oleg Kiselyov's work emphasizes theoretical correctness over ergonomics
