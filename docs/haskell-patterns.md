# Patterns in High-Quality Haskell Codebases

Digesting common idioms and architectural patterns from production Haskell projects.

---

## Core Patterns

### 1. The "Functor Layer" Pattern (Recursion Schemes)

**Where:** Nickel, GHC's Core representation, GHC's AST

**Concept:** Separate the functor layer (one level of recursion) from the recursive type.

```haskell
-- Functor layer - one level of recursion
data TypeF ty
  = App ty ty
  | Forall (Var -> ty)
  | IntTy
  deriving Functor

-- Concrete type - ties the knot
data Type = Type { unType :: TypeF Type }

-- Or with explicit indirection
data Type = Type (TypeF (Fix TypeF))
```

**Benefits:**
- Enables `cata`, `ana`, `hylo` recursion schemes
- Easy to add metadata wrappers (`Spanned Type`)
- Clear separation of structure vs recursion

**Example from nickel:**
```rust
// Rust but same pattern
pub enum TypeF<Ty, RRows, ERows, Te> { ... }
pub struct Type {
    pub typ: TypeF<Box<Type>, RecordRows, EnumRows, NickelValue>,
    pub pos: TermPos,
}
```

---

### 2. Tagless Final Encoding

**Where:** Oleg Kiselyov's work, modern DSLs, GHC plugins

**Concept:** Encode a language using type classes instead of ADTs.

```haskell
-- Language: arithmetic expressions
class Expr repr where
  lit :: Int -> repr Int
  add :: repr Int -> repr Int -> repr Int

-- Multiple interpretations
newtype R a = R { unR :: a }
instance Expr R where
  lit n = R n
  add (R x) (R y) = R (x + y)

newtype P a = P { unP :: String }
instance Expr P where
  lit n = P $ show n
  add (P x) (P y) P $ "(" ++ x ++ " + " ++ y ++ ")"

-- Single program, multiple interpretations
eval :: Expr repr => repr Int
eval = add (lit 3) (lit 4)

-- run: R 5 or P "(3 + 4)"
```

**Benefits:**
- Compile-time extension via new interpreters
- No ADT modification needed
- Type-level enforcement of language features
- Works with Template Haskell for staged interpretation

**Used in:**
- GHC's plugins system
- Streaming library internal DSLs
- Testing frameworks

---

### 3. Bracket Pattern for Resource Management

**Where:** `base`, `resourcet`, `pipes`, `io-streams`

**Concept:** Explicit resource acquisition and release via continuation passing.

```haskell
-- Standard bracket
bracket
  :: IO a           -- Acquire
  -> (a -> IO ())   -- Release
  -> (a -> IO b)    -- Action
  -> IO b

-- Example
withFile :: FilePath -> IOMode -> (Handle -> IO r) -> IO r
withFile path mode = bracket
  (openFile path mode)
  hClose

-- Variant: bracketOnError (only release on exception)
-- Variant: ResourceT (monad transformer)
```

**Benefits:**
- Guaranteed cleanup (even with exceptions)
- Composable resource acquisition
- Type-safe (can't forget to release)

**Variants:**
- `finally`: Always run cleanup
- `onException`: Run cleanup only on error
- `timeout`: Time-bounded actions

---

### 4. Monad Transformer Stack Pattern

**Where:** `mtl` (standard), servant, scotty, yesod

**Concept:** Stack transformers to compose effects.

```haskell
-- Classic stack
type AppM = ReaderT Env (StateT AppState (ExceptT Err IO))

runAppM :: Env -> AppState -> AppM a -> IO (Either Err a)
runAppM env st = runExceptT . evalStateT st . runReaderT env

-- Using mtl type classes (lax monad-control)
class Monad m => MonadReader r m | m -> r where
  ask :: m r
  local :: (r -> r) -> m a -> m a

-- Code is polymorphic over stack
handler :: (MonadReader Env m, MonadError Err m) => m User
handler = do
  env <- ask
  case lookupUser env (userId env) of
    Just u -> pure u
    Nothing -> throwError UserNotFound
```

**Benefits:**
- Effect composition via types
- Code polymorphic over stack order
- Lax type classes (`MonadReader`, `MonadState`) abstract stack

**Trade-offs:**
- Stack reordering requires newtype wrappers
- `MonadTransControl` for operations that need stack access

---

### 5. Van Laarhoven Encoding (lens pattern)

**Where:** `lens`, `generic-lens`, modern optics

**Concept:** Encode optics via function composition.

```haskell
-- Lens type (simplified)
type Lens s t a b = forall f. Functor f => (a -> f b) -> s -> f t

-- Compose lenses: (.) = function composition

-- Example
_1 :: Lens (a, c) (b, c) a b
_1 f (a, c) = (\b -> (b, c)) <$> f a

over :: Lens s t a b -> (a -> b) -> s -> t
over l f = runIdentity . l (Identity . f)

view :: Lens s t a b -> s -> a
view l = getConst . l Const

-- Composing lenses
nestedLens :: Lens S T A B
nestedLens = field1 . field2 . _2
```

**Benefits:**
- No data type needed (encoded in types)
- Composable via `(.)`
- Works with any functor (`Identity` for read, `Const` for write)

**Key insights:**
- `type Traversal s t a b = forall f. Applicative f => (a -> f b) -> s -> f t`
- `type Fold s a = forall m. Monoid m => Getting m s a`
- Hierarchy: `Iso` → `Lens` → `Traversal` → `Fold`

---

### 6. Type Families for Generic Programming

**Where:** GHC's AST, `aeson`, `persistent`, `servant`

**Concept:** Associate types with data constructors.

```haskell
-- Open type family
family Rep a :: *

-- Closed type family (explicit cases)
type family Elem a where
  Elem [a] = a
  Elem (a, b) = a
  Elem a = a

-- Associated type families
class Serialize a where
  type Serialized a :: *
  serialize :: a -> Serialized a
  deserialize :: Serialized a -> a

instance Serialize Int where
  type Serialized Int = ByteString
  serialize = encodeInt
  deserialize = decodeInt

-- Type-level pattern matching
type family Foo a where
  Foo (Maybe a) = a
  Foo [a] = a
  Foo a = a
```

**Benefits:**
- Type-level computation
- Generic serialization (derive via `Generic`)
- Type-safe APIs (Servant routes)

---

### 7. GADTs for Invariants

**Where:** GHC Core, type-safe DSLs, databases (beam)

**Concept:** Indexed data types for correctness.

```haskell
-- Simple GADT
data Expr a where
  LitInt  :: Int -> Expr Int
  LitBool :: Bool -> Expr Bool
  Add     :: Expr Int -> Expr Int -> Expr Int
  If      :: Expr Bool -> Expr a -> Expr a -> Expr a

-- Type-safe evaluation
eval :: Expr a -> a
eval (LitInt n)   = n
eval (LitBool b)  = b
eval (Add e1 e2)  = eval e1 + eval e2
eval (If c t f)   = if eval c then eval t else eval f

-- Compile-time guarantee: only booleans in If condition
-- Type error: If (Add (LitInt 1) (LitInt 2)) (LitInt 3) (LitInt 4)
```

**Benefits:**
- Impossible states unrepresentable
- Type checker as theorem prover
- Clear API boundaries

---

### 8. Source Location Tracking (Spans)

**Where:** GHC, rust-analyzer, nickel, compilers

**Concept:** Attach source positions to every AST node.

```haskell
data Span = Span
  { spanFile :: FilePath
  , spanStart :: (Int, Int)  -- line, col
  , spanEnd :: (Int, Int)
  }

data Located a = Located
  { locSpan :: Span
  , locThing :: a
  }

data Expr a
  = Lit (Located a)
  | App (Located (Expr a)) (Located (Expr a))
  | Lam (Located (Located Name, Expr a))

-- Error reporting
reportError :: Located Expr -> String -> IO ()
reportError (Located span _) msg =
  putStrLn $ spanFile span ++ ":" ++ show span ++ ": error: " ++ msg
```

**Benefits:**
- Precise error messages
- Source mapping (e.g., minification)
- Editor integration (go-to-definition)

---

### 9. Multi-Pass Compiler Pipeline

**Where:** GHC, rustc, compilers

**Concept:** Pipeline of transformation passes.

```haskell
-- Stages of compilation
data Pipeline
  = Parse
  | Rename
  | TypeCheck
  | Desugar
  | CoreOptim [Pass]
  | STG
  | Cmm
  | Asm

-- Pass type
type Pass a b = a -> Either CompilerError b

-- Pipeline composition
runPipeline :: Pipeline -> Source -> Either CompilerError Asm
runPipeline Parse src = parse src
runPipeline Rename ast = rename ast
runPipeline TypeCheck ast = typecheck ast
runPipeline Desugar ast = Right $ desugar ast
runPipeline (CoreOptim passes) core = foldM ($) core passes

-- Each pass is pure (monad for logging/state)
simplify :: Pass Core Core
simplify = betaReduce . inline . constFold
```

**Benefits:**
- Composable passes
- Easy to add/insert passes
- Pure transformations testable

---

### 10. Generic Derivation via Generics

**Where:** `aeson`, `lens` (`makeLenses`), `quickcheck`

**Concept:** Derive boilerplate from structure.

```haskell
{-# LANGUAGE DeriveGeneric #-}
import GHC.Generics

data User = User
  { userName :: String
  , userAge :: Int
  } deriving (Generic)

-- Generic implementation
instance ToJSON User where
  toJSON = genericToJSON defaultOptions
  toEncoding = genericToEncoding defaultOptions

-- Customizing field names
data User = User
  { _userName :: String
  , _userAge :: Int
  } deriving Generic

deriveLenses ''User  -- Template Haskell

-- Or via Generics (no TH)
deriveLensesGeneric ''User
```

**Benefits:**
- Reduce boilerplate
- Consistency (all fields handled)
- Type-safe (compile-time)

---

### 11. STG (Spineless Tagless G-machine) Pattern

**Where:** GHC's runtime representation

**Concept:** Represent lazy evaluation via explicit applications.

```haskell
-- STG language (simplified)
data StgExpr
  = StgApp Id [StgArg]
  | StgLit Literal
  | StgCon Id [StgArg]

data StgArg
  = StgVarArg Id
  | StgLitArg Literal

-- Evaluation: only evaluate when pattern matching
eval :: StgExpr -> Value
eval (StgApp f args) =
  case lookupEnv f of
    Closure env body -> eval (subst args env body)
eval (StgLit l) = literalToValue l
```

**Benefits:**
- Explicit lazy evaluation
- Efficient heap representation
- Control over evaluation order

---

### 12. Concurrency Patterns

#### STM (Software Transactional Memory)
```haskell
type STM a  -- Transaction monad

-- Transactional variable
type TVar a  -- Shared, transactional

-- Operations
readTVar :: TVar a -> STM a
writeTVar :: TVar a -> a -> STM ()
retry :: STM a  -- Retry transaction if blocked
orElse :: STM a -> STM a -> STM a

-- Running transactions
atomically :: STM a -> IO a
```

**Benefits:**
- Composable concurrent operations
- No deadlocks (retry semantics)
- Optimistic concurrency

#### Async Pattern
```haskell
-- Spawn async action
async :: IO a -> IO (Async a)

-- Wait for result
wait :: Async a -> IO a

-- Resource-safe async
withAsync :: IO a -> (Async a -> IO b) -> IO b

-- Race multiple asyncs
race :: IO a -> IO b -> IO (Either a b)

-- Concurrently run all
concurrently :: IO a -> IO b -> IO (a, b)
```

**Benefits:**
- Structured concurrency (bracket pattern)
- Cancellation support
- Composable async operations

---

### 13. Test Property Patterns

#### QuickCheck
```haskell
-- Arbitrary: random generation
class Arbitrary a where
  arbitrary :: Gen a
  shrink :: a -> [a]

-- Property: first-class test
newtype Property = Property { unProperty :: Gen Result }

-- Example property
prop_reverse :: [Int] -> Bool
prop_reverse xs = reverse (reverse xs) == xs

-- Test
quickCheck prop_reverse
```

**Key insight:** Shrinking finds minimal counterexamples.

#### Hedgehog
```haskell
-- Integrated shrinking
genInt :: Gen Int
genInt = Gen.int (Range.linear 0 100)

-- Property
prop_sum :: Property
prop_sum = property $ do
  x <- forAll genInt
  y <- forAll genInt
  (x + y) === (y + x)
```

**Benefits:**
- Shrinking built-in
- State machine testing
- Composable generators

---

### 14. Parser Combinator Pattern

#### Megaparsec
```haskell
-- Parser type
type Parser = Parsec Void Text

-- Combinators
char :: Char -> Parser Char
string :: String -> Parser String
many :: Parser a -> Parser [a]
sepBy :: Parser a -> Parser b -> Parser [a]

-- Error context (<?>)
expr :: Parser Int
expr = term `sepBy` char '+' <?> "expression"

-- Try for backtracking
statement :: Parser Stmt
statement = try (parseIf) <|> parseLet <|> parseAssign
```

**Benefits:**
- Composable
- Good error messages
- Backtracking control

---

### 15. Effect Systems (Modern)

#### Polysemy-style effects
```haskell
-- Effect: operation set
data Reader r m a where
  Ask :: Reader r m r

-- Make it an effect
makeSem ''Reader

-- Program: polymorphic over effects
program :: forall r. Member (Reader Int) r => Sem r Int
program = do
  n <- ask
  pure (n + 1)

-- Interpret: run effect
runReader :: Int -> Sem (Reader Int ': r) a -> Sem r a
runReader env = interpret \case
  Ask -> pure env
```

**Benefits:**
- Composable effects
- Order-independent
- Type-safe

---

## Architectural Patterns

### 1. Service Layer Pattern

**Where:** web apps, microservices

```haskell
-- Service typeclass
class Monad m => UserService m where
  getUser :: UserId -> m (Maybe User)
  createUser :: CreateUserRequest -> m User

-- Implementation
instance UserService (AppM Env) where
  getUser uid = runDB $ get uid
  createUser req = runDB $ insert req

-- Handler uses service interface
handleGetUser :: (MonadError AppError m, UserService m) => UserId -> m User
handleGetUser uid =
  getUser uid >>= maybe (throwError UserNotFound) pure
```

**Benefits:**
- Testable (mock services)
- Swappable implementations
- Clear boundaries

---

### 2. Repository Pattern

**Where:** database access, persistence

```haskell
-- Repository interface
class Repository a where
  type Key a
  find :: Key a -> AppM (Maybe a)
  findAll :: AppM [a]
  save :: a -> AppM ()
  delete :: Key a -> AppM ()

-- Database implementation
instance Repository User where
  type Key User = UserId
  find uid = runDB $ get uid
  findAll = runDB selectList [] []
  save user = runDB $ insert_ user
  delete uid = runDB $ delete uid

-- Handler polymorphic over repository
handleGetUser :: Repository a => Key a -> AppM (Maybe a)
handleGetUser = find
```

**Benefits:**
- Abstract persistence
- Easy to mock
- Swappable backends

---

### 3. Middleware Pattern

**Where:** WAI, servant, scotty

```haskell
-- Middleware type
type Middleware = Request -> (Response -> IO ResponseReceived) -> IO ResponseReceived

-- Logging middleware
logging :: Middleware
logging app req respond = do
  putStrLn $ method req ++ " " ++ path req
  app req respond

-- Compose middlewares
app :: Application
app = logging (compress (csrf myHandler))
```

**Benefits:**
- Cross-cutting concerns
- Composable
- Testable

---

### 4. Plugin/Extension Pattern

**Where:** GHC, HLS (Haskell Language Server)

```haskell
-- Plugin type
data Plugin = Plugin
  { pluginName :: String
  , pluginTypecheck :: CoreModule -> CoreM CoreModule
  , pluginRewrite :: Expr -> Maybe Expr
  }

-- Load plugins
loadPlugins :: [FilePath] -> IO [Plugin]
loadPlugins = mapM loadPlugin

-- Run plugins
runPlugins :: [Plugin] -> CoreModule -> CoreM CoreModule
runPlugins plugins = foldM (\m p -> pluginTypecheck p m)
```

**Benefits:**
- Extensible without modification
- Third-party contributions
- Dynamic behavior

---

## Anti-Patterns to Avoid

1. **Overly clever type-level hacks** - code golf vs readability
2. **Huge monad stacks** - hard to debug, stack overflow
3. **Excessive `IO`** - leaks impurity
4. **Partial functions** - runtime panics
5. **String everywhere** - use `Text` or `ByteString`
6. **Ignoring performance** - space leaks, boxed types
7. **Over-engineering** - YAGNI, keep it simple

---

## When to Use Each Pattern

| Pattern | Use when |
|---------|----------|
| Tagless final | Multiple interpretations, extensible DSL |
| Functor layer | Tree/AST manipulation, recursion schemes |
| Bracket pattern | Resource management |
| Monad transformers | Composable effects |
| Van Laarhoven | Optics, composable getters/setters |
| GADTs | Type-safe invariants, indexed types |
| Source location | Compilers, editors |
| Multi-pass pipeline | Compilers, data processing |
| Generics | Boilerplate reduction |
| STM | Composable concurrency |
| Async | Structured concurrency |
| Service/Repository | Clean architecture, testing |
| Middleware | Cross-cutting concerns (auth, logging) |

---

## Resources for Learning

- **GHC**: Read `GHC.Core`, `GHC.Tc`, `GHC.Driver`
- **lens**: Type signatures are the documentation
- **pipes**: Tutorial in README
- **servant**: Type-level routing examples
- **QuickCheck**: `test-framework` documentation
- **Haskell Wiki**: Design patterns
- **School of Haskell**: Tutorial articles
- **Haskell Symposium Papers**: Academic patterns
