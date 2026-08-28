# Current RoadMap

_Updated 26-08-28_

## Things that we want done 

The goal of this document is to collate what features exist in docs and roadmaps and which ones still need implementation. Go through this and cross-reference the files referenced and the code to figure out status on them and then, in this document, at the end, create a new todo-list of things to do in a suggested "good order" to do it in. 

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

What would be nice is a slicker transitioning of the game, in general. I assume we could simply iterate over things, but I have some thoughts already - after each phase in the game, we should be given a summary of what happened, because this is open information in the game.

#### Phases and Summaries

The current phase should be shown clearly in the pane for it.

1. Taxation - what every player paid in taxes, very simple, and if they used coinage to increase or decrease taxation (selected before taxation occurs) - and of course, if any city revolts occured. 
2. Population Expansion - our tokens aren't exactly perfectly positioned on the board. If there are multiple tokens, we should indicate this. We have some indicators surrounding areas, but some visual indicator that there are a **lot** of tokens in an area is helpful. I think there is an algorithm that shifts tokens slightly if they are more than one, but I think this information is thrown away in some circumstances - and also, tokens of different civilizations should not stack together, they should form independent piles.
3. Census is not necessary to make a big deal of, it updates the list in the game, done.
4. Ship construction - which player built a ship, where. Did they maintain a ship? Did they scrap a ship?
5. Movement - when the player is done, a replay could potentially be made showing every move on the board, otherwise... this one would become a lot of information without value quickly.
6. Conflict - great to have a list of conflicts that occured, number of tokens on each list and outcome (i.e. end number of tokens, since both players can co-exist after a battle).
7. Who and where are cities built?
8. No need to list surplus removal - but perhaps if someone loses a city it could be interesting information?
9. Who draws how many cards? 
10. After all trade is said and done, a summary of trades made could be done - only revealing open cards, of course.
11. Calamity Resolution, this one is great - who got hit by what where and did they have cards to mitigate it?
12. Summary of who acquired what cards, simple. - Also, a note here, hovering or click a civ card should show it's entire card text / description and credits - and hey - when selected, all other non-purchased cards should be highlighted with their new price **with the selected card** - making it easier to take discounts into account when purchasing.
13. Movement on AST - super important. If a player cannot move, he must be made aware of it. If some else moves, that should be highlighted as well.

Clearly, this can be done in so many different ways. In the Sid Meier Games, for instance, if the player gets attacked, we zoom in on that attack. This happens here as well, but there is lag to it. I could live with having the camera pan to areas of interest if something happens, that is a fine idea, but it has to **work**. If we do some kind of addition to the Game Info / Round Info pane, we could have some slight delay of events to slow the game down a bit so the player can absorb the info. So, say we are doing movement and the player is 3rd, then we list all the previous players moves in that pane... OR show them as they happen on the board... Suggestions welcome, as always we do it easy and clearly first, then improve later.