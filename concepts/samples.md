# Sample program in Folidity

Sampled as part of the design of BNF grammar.

Let's design an on-chain commit-reveal voting smart contract

```rust
// metadata is in the header

// version of the compiler
version: "1.0.0"
author: Gherman Nicolisin <gn2g21@soton.ac.uk>

// This is a comment.

enum Choice {
    None,
    Yay,
    Nay
}

// We can define models separately from state and reuse them
model BeginModel {
    start_block: int,
    end_block: int,
    // voters are the set, so only unique entries are allowed
    voters: Set<Address>,
    // let's represent the proposal as string for now
    proposal: String,
    // describe the max size of the voters list
    max_size: int,
} st { // `st` keyword means "such that" used to indicate the model constraints
    // voting start should be greater than current block by 10 blocks
    // `current_block` is part of embedded model state
    start_block > (current_block + 10),
    end_block > (start_block + 10),
    //voters balance should greater than 1000 tokens
    voters.balance > 1000,
    max_size > 0,
    voter.size <= max_size
}

// Voting model extends `BeginModel` and its guards
model VotingModel: BeginModel {
    commits: Mapping<Address, Hash>,
} st {
    // Specify the type of mapping
    commits: Address -> int, // `->` means one-to-one relationship
    // voter must be in the set of voters
    commits.key in voters,
}

// Describes how reveal should work
model RevealModel {
    proposal: String
    end_block: int,
    // we only interested in commitments at this stage
    commits: Mapping<Hash, Choice>,
} st {
    end_block > (current_block + 15),
    yays >= 0,
    nays >= 0,
    // we set 1-1 relation between 
    commits: Hash -> Choice,
    //total sum of `yays` and `nays` must not exceed the total commits size
    (yays + nays) <= commits.size
}

// Described model for the execution
model ExecuteModel {
    proposal: String,
    passed: bool
}


// Let's describe states now

state BeginState(BeginModel);
state VotingState(VotingModel);

// Reveal state has explicit constraints that it must be transition from `VotingState`
state RevealState(RevealModel) from VotingState st {
    // we specify that we can transition into this state only when
    current_block > VotingState.end_block
};

state ExecuteState(ExecuteModel) from RevealState st {
    current_block > RevealModel.end_block 
};
// can be equally described as
state ExecuteState {
    proposal: String,
    passed: bool
} from RevealState {
    current_block > RevealModel.end_block 
}


// messages are reason as events

// `any` means anyone can call this function
@(any)
@init
fn () init(proposal: String, 
          start_block: int, 
          max_size: int, 
          end_block: int) 
when () -> BeginState
{
    BeginState {
        proposal,
        start_block,
        end_block,
        max_size
    }
}

@(any)
fn () join() when BeginState s -> BeginState {
    // `caller()` is the built-in function
    let caller = caller();

    // let's decompose the state
    let { voters, params } = s;

    voters += caller;

    BeginState {
        voters,
        // fill other fields
        ..params,
    }
}

// `@()` accepts a `Set<Address>` or `Mapping<Address, X>` or `List<Address>`
// sets can be combined
// `@(X | Y | Z)`
@(voters)
fn () start_voting() when BeginState -> VotingState {
    commits = new Set();
    BeginState(model) -> VotingState(commits, model)
}

@(voters)
fn () commit(h: Hash) when VotingState s -> VotingState {
    let caller = caller();
    let { commits, params } = s;

    commits.set(caller, h);

    // symbolic execution will highlight
    // that model is violated since you are trying to add the caller twice
    // if (caller.balance > 2000) {
    //     commits.set(caller, 0); 
    // }

    VotingState {
        commits.
        ..params
    }
}

@(any)
fn () start_reveal() when VotingState -> RevealState {
    VotingState { end_block, proposal, commits, params } 
        -> RevealState {
            end_block = endblock + 10,
            proposal: proposal,
            commits: commits.map(|c| (c.value, Choice::None))
            yays: 0,
            nays: 0,
        }
}

@(any)
fn int reveal(salt: int, vote: Choice) 
when RevealState s1 -> (RevealState s2, ExecuteState s3) // we can transition to 2 different states
// we ensure that the size of mapping doesn't change
st s1.commits.size == s2.commits.size
{
    let calc_hash = hash(caller(), vote, salt);
    let { commits, params } = s1;
    s.set(commits, vote)

    if s1.current_block > s1.end_block {
        execute()
    } else {
        RevealState {
            commits,
            ..params
        } return commits.size // return integer values after state transition
    }
}

@(any)
fn () execute() st RevealState s -> ExecuteState {
    let votes = s.commits.values;
    let yay = votes.filter(|v| v == Choice::Yay).sum();
    let mut passed = false;
    if (votes.size / yay) > 0.5 {
        log(f"Proposal {s.proposal} passed");
        passed = true;
        ExecuteState {
            proposal: s.proposal,
            passed
        }
    } else {
        log(f"Proposal {s.proposal} not passed");
        ExecuteState {
            proposal: s.proposal,
            passed
        }
    }
}


```