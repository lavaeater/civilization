# Current RoadMap

_Updated 26-08-28_

## Things that we want done 

### Agent API Design

See _agent-api-design.md_

## Follow-ups

- Implement Trade resolution (offer/accept/settle) end-to-end, then expose it.
- Civ-card purchase: compute cost + payment so `AcquireCard` is applyable.
- Auth/port config if this is ever exposed beyond localhost.


### AST Design

See _ast-design.md_

#### 7. Known follow-ups (not in scope here)

- Full victory scoring (rule 35) and a real game-over state — only the FINISH
  *trigger* (rule 34.1A) is logged today.
- Per-civ Late-Iron point values / age-length overrides mined from the "ASTCalc"
  sheet, dropped into `AstTrack::overrides`.
- Marker positioning currently assumes the standard 17-cell geometry; revisit if
  per-civ overrides change the cell count.
  </content>

### Enhanced Input

See _enhanced_input.md_

- [ ] Step 5 — Phase contexts (incremental, one at a time)
- [ ] Step 6 — Universal Confirm / Cancel

### Multiplayer

See _multiplayer.md_ and _running-multiplayer.md_

#### Notes on Multiplayer

That document is pretty large, so I didn't scan it for what is done and not done, what I do know is that an Android / iOS app is super not prioritized. The game is very much a desktop / web game, i.e. being played on a computer. 

Second, my goal is that the game will have a running server using my pingora setup (~/projects/rust/dioxus/pingora-docker), so it will simply have an entry in that docker-compose file, and an entry in the services.json file so that pushing a tag to the repo rebuilds it. I have control of the machine that the server is running on so that is not a problem, all of the things running there I have control over - which also means that we could implement a server-side save-game strategy - either by saving to files or even a postgres database if needed. 

My ideal MVP scenario for online multiplayer is that we have one player that is admin and can access the game to create a multiplayer game. A multiplayer game gets a magic link, navigating to that link connects a player to the game, lets them select faction and player name. Reconnecting is as simple as going to that magic link again, I guess. Easy stuff. And then the human players can start playing.

### Utility AI Design

See _utility-ai-design.md_

There are some things not yet implemented and / or tested in there, we should analyze them. My thoughts on the AI is to make the AI very "pointed" towards different ways of winning the game. For instance, conquest is a viable strategy, but so is trade... so perhaps one cannot simply reduce a strategy in this type of game to simple playstyles... but I wouldn't rule it out, right? 

What I have noticed is that the player in general is not keen enough on building cities. Regardless of playstyle, without cities, a player is doomed. The goal of every player is to advance on the AST, reach the end and then score the highest using the games scoring protocols. Getting there fast can perhaps debilitate other players to not score highly, I am not sure. But without cities, advancement on the AST is impossible - which concretely means that an AI must check their ability to advance every now and then. As a human player I am keenly aware of this at all times - but as a note on UI in general, when moving ahead on the AST, this could be noted.

| Playstyle    | Character | Knobs that dominate |
|--------------|-----------|---------------------|
| `Balanced`   | the reasonable default | everything ~0.5 |
| `Warlord`    | attacks, contests cities, low reserves | `aggression`, `risk` high; `defense` low |
| `Expansionist` | grabs land fast, thin everywhere | `expansion`, `growth` high; `defense` low |
| `Builder`    | cities + upkeep, hard to dislodge | `city_income`, `defense` high; `aggression` low |
| `Merchant`   | farms trade cards & civ tech | `trade_drive`, `tech_focus` high |
| `Turtle`     | minimal footprint, never overextends | `defense`, `calamity_aversion` high; `risk` low |

### New Notes on Gameplay and Transitions

The game is pretty near feature completion. I am not entirely sure all calamities work as they should (Barbarian Hordes is one, I think we must work more on), but this can be explored and tests can be written.