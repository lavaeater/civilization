# Dialog-based trade

Status: step 1 (rules, data, AI adapter, minimal human controls) built on
branch `trade-dialog`; steps 2–4 to do. Replaces the "open offer anyone can accept"
model in `concepts/trade` with a table conversation that works like trading
around a real board.

## The idea

At a real table people mostly shout what they want, answer with what they
have, haggle, and undercut each other. Every statement is a bid; nobody can
simply take someone's open offer. A deal is always the end of an
offer / counter-offer exchange, and while two players haggle, a third can jump
in and sweeten the deal.

## Two kinds of statement

**Talk** is free and never binding, and it may be a lie ("I have Wine" proves
nothing):

| Kind | Example |
|---|---|
| Want | "I want Iron!" (starts a thread) |
| Reply | "I have Iron; I want Grain, Wine or Salt — got any?" |
| Decline | "No, I want Grain and Salt myself." (reason optional) |

**A proposal** is a complete deal that one player can accept. It must satisfy
rule 28.3: at least 3 cards each side, and the 2 named cards on each side are
truthful and checked against the proposer's hand. Hidden cards and claims
("...and two more Salt") stay bluffable, exactly as today. A proposal may also
carry a non-binding *request* for the other side's hidden card ("you can give
me Hides as the third").

**Every proposal is addressed to exactly one player, and only that player can
accept it.** Anyone may still post their own proposal to the same player; that
is how a third player undercuts or sweetens.

## Example session

| Who | Message | Kind |
|---|---|---|
| A | I want Iron | Want (new thread) |
| B → A | I have Iron; I want Grain, Wine or Salt | Reply |
| A → B | No, I want Grain and Salt myself | Decline |
| C → B | I have Wine and Salt, got Oil? | Reply |
| B → C | My 2 Oil + 1 hidden for your Wine + Salt + 1 hidden | Proposal to C |
| D → B | My Wine + Salt + 1 hidden ("it's Salt") for your 2 Oil + 1 hidden ("Hides would be nice") | Counter-proposal to B |
| B | Accepts D's proposal | → settlement |

Once B accepts, any other live proposal from or to B that B's (or the other
party's) hand can no longer cover is withdrawn automatically.

## Decisions (recommended defaults, to confirm)

1. Talk is never binding. Only proposals bind, and only on the named cards.
2. A proposal is addressed to one player, never to "anyone in the thread".
3. A player may have several proposals out at once. Accepting one withdraws
   the others it makes impossible to fulfil.
4. Declines take an optional reason; quick buttons cover the common ones.

## Screen

- **Table feed**: messages grouped by thread, newest activity first, each line
  showing who said it to whom.
- **Composer**: "I want…", "I have…", or a full proposal. Clicking a message
  starts a reply pre-filled from it: clicking "I have Wine and Salt, got Oil?"
  opens a proposal to that player with those cards filled in.
- **Accept / Decline** only on proposals addressed to you, each showing the
  hand-value line (value before → after, net) already used in the offer dialog.
- The trade clock pauses while a lone human composes or settles (already done).

## AI

The valuation and offer planning in `trade_functions.rs` / `stupid_ai/scoring/
trade.rs` carry over. New is taking part in the conversation:

1. Announce wants from its hand.
2. Answer wants it can fill with "I have X, I want Y".
3. Turn a promising reply into a proposal.
4. Judge proposals addressed to it with today's acceptor scoring; accept or
   decline.
5. Undercut or sweeten in others' threads. Personalities show here: Merchants
   jump in, Turtles mostly answer.

Wants reveal hands, so an AI can later remember "C wants Oil".

Pacing: each AI speaks at most every 1–2 s and talk per thread is capped, so
nine AIs still produce a readable feed.

## Build steps

1. **Rules and data**: messages, threads, addressed proposals, accept-only-by-
   addressee, withdrawal of proposals that can no longer be fulfilled; unit
   tests. `OpenTradeOffer` becomes a proposal and the accept-anything button
   goes. AIs get a simple adapter: post wants, propose to whoever replies,
   accept or decline what is addressed to them.
   *Done:* `trade_talk.rs` (`TradeTalk`, `TalkKind`, `TableKnowledge`,
   `ai_trade_talk`, `withdraw_unfulfillable_proposals`, `clear_withdrawn_offers`,
   `prune_old_talk`, `[TALK]` log lines); `OpenTradeOffer::can_accept` only for
   the addressee, `thread` field; AI proposals pick (deal, partner) by talk fit;
   AIs decline unwanted proposals after 1.5 s; the offer dialog has a "Proposal
   to" chooser; Accept/Decline only on proposals to you; the feed shows talk
   (others' declines hidden); `/trade/offer` requires `target`.
2. **Human screen**: feed and composer with pre-filled replies.
3. **AI conversation**: replying, countering, undercutting, declining; compare
   self-play pass-on rate and trade value with today.
4. **Later**: multiplayer — messages are moves, so they fit the move-based
   protocol in `docs/multiplayer.md`.
