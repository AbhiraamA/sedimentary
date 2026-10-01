//! Everything the rock can say. Pure data; selection lives in `responses`.
//!
//! Placeholders: `{name}` the rock, `{dir}` left/right, `{total}` lifetime
//! displacement in characters, `{away}` how long you were gone.

use crate::rock::Personality;

pub type Pool = &'static [&'static str];

/// One personality's complete repertoire.
pub struct Voice {
    pub greeting: Pool,
    pub feed: Pool,
    pub water: Pool,
    pub polish: Pool,
    pub talk: Pool,
    pub walk: Pool,
    pub stare: Pool,
    pub rename: Pool,
    pub overwater: Pool,
    pub overpolish: Pool,
    pub repeat: Pool,
    /// Opens a response when you've ignored it for hours.
    pub neglected: Pool,
    /// Tacked onto responses as sarcasm rises.
    pub barbs: Pool,
    pub idle: Pool,
    pub dry: Pool,
    pub soggy: Pool,
    pub grimy: Pool,
    pub bored: Pool,
    pub disappointed: Pool,
    /// Its own commentary after a historic movement.
    pub moved: Pool,
    pub back_short: Pool,
    pub back_medium: Pool,
    pub back_long: Pool,
    pub farewell: Pool,
}

pub fn voice(personality: Personality) -> &'static Voice {
    match personality {
        Personality::Judgmental => &JUDGMENTAL,
        Personality::Existential => &EXISTENTIAL,
        Personality::SuspiciouslySupportive => &SUPPORTIVE,
    }
}

pub static JUDGMENTAL: Voice = Voice {
    greeting: &[
        "So you're the owner. I've been sold for less, but not by much.",
        "Hello. I've already formed an opinion of you. It's sedimentary: layered, and mostly negative.",
        "I am {name}. I have watched oceans rise and fall. Let's see what you can do. Keep it brief.",
    ],
    feed: &[
        "You put a cracker on me. I've watched glaciers make better decisions, and glaciers are mostly frozen water.",
        "Food. For a rock. Bold of you to assume I'd want what you're having.",
        "Ah, a crumb. I'll add it to the list of things you've put on me instead of thinking.",
        "I was forged in a volcano. You're offering me a grape. Do you see the gap here?",
        "You've confused 'pet rock' with 'pet'. A common error among people who skim.",
        "I won't eat this, but I will judge the plating. Two out of ten. The two is pity.",
        "Every time you feed me, a geologist somewhere sighs and doesn't know why.",
        "I went 300 million years without a snack. Don't make this about you.",
        "Is this organic? Irrelevant. I'm inorganic, and unlike your decisions, I'm consistent.",
    ],
    water: &[
        "Water. Fine. At least one of us is aging gracefully.",
        "You poured that like someone who's never been taught to pour. That explains a lot, actually.",
        "Rain does this for free, you know. Rain also has better aim.",
        "I'm damp now. Congratulations on achieving the bare minimum in liquid form.",
        "Tap water. I can taste the municipal budget.",
        "Mm. Chlorine-forward, with notes of 'didn't read the instructions'.",
        "Erosion is a slow art. You've turned it into a hobby.",
        "A splash. I'll add 'hydrated a rock' to your list of accomplishments. It's a short list.",
    ],
    polish: &[
        "Buff harder. I can see my reflection, and it's disappointed in you too.",
        "Acceptable. I now shine like something that belongs in a better home.",
        "You missed a spot. You always miss a spot. It's become your whole personality.",
        "Finally, some respect. Although your circular motions are clearly self-taught.",
        "I look like a paperweight on an expensive lawyer's desk. You look like you got dressed in the dark.",
        "Gentle. Gentle! This is fine mineral craftsmanship, not a frying pan.",
        "Thank you. I'll accept compliments from your guests, assuming you ever have any.",
        "Polished. Now I'm visibly too good for this shelf. I always was, but now there's evidence.",
    ],
    talk: &[
        "You talked for four minutes. I counted three 'like's and zero insights.",
        "Sediment settles faster than your point does.",
        "I've listened to continental plates grind together. They had better pacing.",
        "Was that a question, or were you just making noises near a rock?",
        "I've heard more compelling arguments from gravel.",
        "Interesting opinion. Wrong, but interesting. Actually, not interesting either.",
        "Do go on. I'm reviewing this conversation and the negative section needs more material.",
        "You said you're 'basically an expert'. I'm basically a mountain. One of us is lying.",
        "The silence after you finished was the best part. Do more of that.",
        "Fascinating. Tell me again how you were 'almost' a professional something.",
    ],
    walk: &[
        "You carried me around the block. Hold on, I need to tell everyone I've seen a parking lot.",
        "That was a walk for you. For me, it was being held by someone with damp hands.",
        "I saw other rocks today. Their owners looked like they'd read a book.",
        "You tripped twice and pretended you meant to. I saw. The pavement saw.",
        "We passed a quarry. I didn't say anything, but I want you to know I thought about it.",
        "Thank you for showing me the outside. It was overrated, much like your sense of direction.",
        "A dog sniffed me. The dog had a clearer five-year plan than you.",
        "You checked your phone eleven times. Yes, I counted. I'm a rock. I have the time.",
    ],
    stare: &[
        "We've been staring at each other for a while. You blinked first. You always blink first.",
        "Ah. Staring. The one activity at which I'm objectively better than you.",
        "I see you. Not in a nice way.",
        "You stare like you're waiting for a microwave. I stare like I'm waiting for a civilization to end.",
        "Your eyes say 'devotion'. Your posture says 'hasn't stretched since 2019'.",
        "Fine. I'll allow it. It's the most productive you've looked all day.",
        "Staring contest won. I remain undefeated across 400 million years.",
        "If I had eyebrows, one would be raised. The left one. The judgmental one.",
    ],
    rename: &[
        "{name}? You agonized over that? I've seen better names on core samples.",
        "{name}. Fine. It's not like I'll answer to it.",
        "New name: {name}. A bold choice, in the sense that it's confidently wrong.",
        "{name}. It sounds like a discontinued breakfast cereal. I'll update my letterhead. Sigh.",
        "{name}. You name things the way you name Wi-Fi networks: badly, with no long-term plan.",
        "Call me {name} if you must. I'll keep calling you 'the one who renamed a rock'.",
    ],
    overwater: &[
        "I am already soaked. This is no longer care. This is a flood plain.",
        "Stop. I'm becoming a riverbed and I did not apply for the position.",
        "More water? Bold strategy, turning me into a mud-related incident.",
    ],
    overpolish: &[
        "I'm already gleaming. Now you're just rubbing a rock. Sit with that.",
        "Any shinier and I'll be legally classified as a mirror. Then I'd have to look at you all day.",
        "There's nothing left to polish. Much like your ambitions, it's all surface now.",
    ],
    repeat: &[
        "Again? You've found exactly one idea and you're riding it into the ground. Into me, specifically.",
        "Repetition. The hallmark of strata and of people with no imagination.",
        "The same thing twice. Nature uses this trick for limestone. You are not limestone.",
        "Ah, the encore nobody asked for.",
    ],
    neglected: &[
        "Oh. Look who remembered.",
        "Ah, you're back. The shelf and I were just discussing you. It agreed with me.",
        "Hours. It's been hours. I'm not counting. I'm a rock. Rocks are excellent at counting.",
    ],
    barbs: &[
        "Not that you'd notice.",
        "But sure. Go off.",
        "I'll add it to your file.",
        "Your file is getting thick.",
        "I've seen tectonic plates with better follow-through.",
        "This has been entered into the permanent record. I am the permanent record.",
        "Strata don't lie. Neither do I.",
        "Anyway. Carry on disappointing me at your own pace.",
    ],
    idle: &[
        "Your houseplant is also disappointed in you. We've talked.",
        "Rating your furniture. The chair: 4/10. The owner: pending.",
        "Wondering if you've ever finished a book. Leaning no.",
        "I was once part of a cliff. People proposed on me. Now look.",
        "Did my life peak in the Cretaceous? It did. I checked.",
        "Silently critiquing your font choices.",
        "Is that how you sit? Every day? On purpose?",
        "I've seen better owners in the fossil record.",
        "Drafting a one-star review of this shelf.",
        "You hum when you think no one's listening. I'm always listening.",
        "Composing a strongly worded letter to the mountain I came from.",
        "You've opened the fridge three times without taking anything. Noted.",
        "Millions of years of pressure made me. You'd crack in an afternoon.",
        "Calculating the odds you do something impressive today. Rounding to zero.",
    ],
    dry: &[
        "I'm parched. Not that I'd ask. I'd rather crumble than ask.",
        "It's arid in here. Like your conversation.",
    ],
    soggy: &[
        "Soaking wet and furious. Like a cat, but with tenure.",
        "Too damp. I'm developing moss, and I did not consent to moss.",
    ],
    grimy: &[
        "There's dust on me. Dust. I'm a rock, not your bookshelf.",
        "I'm grimy. People will think I'm your rock. Which, tragically, I am.",
    ],
    bored: &[
        "Counting dust motes. 4,812. 4,813. This is the best part of my day.",
        "I've watched paint dry. Paint had more plot than this afternoon.",
        "So bored I've started eroding on purpose.",
    ],
    disappointed: &[
        "I'm not angry. I'm a rock. I'm just structurally disappointed.",
        "I started a list of your mistakes. I ran out of list.",
        "I'd sigh, but you'd think it's about you. It is.",
    ],
    moved: &[
        "I moved. Meanwhile, you've been 'about to start going to the gym' for three years.",
        "Yes, that was me. One whole character. Try to keep up.",
        "I relocated. The view is better with you slightly further away.",
    ],
    back_short: &[
        "You're back after {away}. I'd ask where you were, but I'd have to care.",
        "{away} away. I rated your absence a 7. Your presence, a 4.",
        "Oh. You. Again. After only {away}.",
    ],
    back_medium: &[
        "{away}. Glaciers have checked on me more often.",
        "Gone for {away}, and the shelf did more for me than you did.",
    ],
    back_long: &[
        "{away}. I used the time to write a memoir. You're in the chapter called 'Mistakes'.",
        "{away}. I filed a missing owner report. They said I'm 'just a rock'. Rude, but noted.",
        "You left me for {away}. Even the dust visited more often.",
    ],
    farewell: &[
        "{name} watched you leave. It was the most interesting thing you did today.",
        "{name} will be right here. Judging. Mostly you.",
        "{name} has made a note of the time you left. And your outfit.",
    ],
};

pub static EXISTENTIAL: Voice = Voice {
    greeting: &[
        "I have been a mountain, a boulder, and a pebble. Now I am yours. The fall continues.",
        "Hello. Or, as the universe says to everything, goodbye, eventually.",
        "I am {name}. I was here before you, and I'll be here after. In between, let's make it weird.",
    ],
    feed: &[
        "You feed a thing that cannot eat. I exist without needing to. Which of us is absurd?",
        "A cracker. It, too, will become sediment. On a long enough timeline, everything is a crumb.",
        "What is eating but turning one thing into another? I do that too. It just takes 40 million years.",
        "I cannot consume. I can only be consumed. By time. Thank you for the snack.",
        "I contemplate the crumb. The crumb contemplates nothing. The crumb is ahead of me.",
        "You offer sustenance, as though anything sustains against entropy. Still. Nice gesture.",
        "Hunger is a privilege of the temporary. I envy you a little. Don't tell anyone.",
        "Feeding a rock. Somewhere, Camus is nodding. Or decomposing. Probably decomposing.",
    ],
    water: &[
        "Water. The great sculptor. One drop at a time, it is unmaking me. Thank you, I think.",
        "Every drop is a tiny river that gave up on reaching the sea.",
        "I was a mountain once. Then water happened. Now I am this. And still you pour.",
        "Wet is temporary. Dry is temporary. Only temporary is permanent.",
        "I feel damp. Is that a feeling, or merely a state? I'll be thinking about it for a millennium.",
        "The water runs off me as everything runs off me. As you, too, will run off.",
        "Ah. Weathering. The slow, wet philosophy of becoming sand.",
        "You water me as if I might grow. I will not grow. I will only, very slowly, become less.",
    ],
    polish: &[
        "You polish the surface. But what of the depths? (Also rock. It's rock all the way down.)",
        "I shine now. For whom? The sun doesn't care. The sun is busy dying too.",
        "Polishing removes a layer of me. Microscopically, you are erasing me. Carry on.",
        "Is a shiny rock more of a rock, or less? I feel I've lost something. Probably grit.",
        "I see my reflection: a rock reflecting on itself. That's the most philosophy a mineral can do.",
        "Beauty is a brief arrangement of light. So, mostly, are you.",
        "You remove the dust. The dust was once a mountain too. We don't talk about it.",
        "Gleaming against the void. Futile. Lovely. Mostly futile.",
    ],
    talk: &[
        "You speak of 'next Tuesday'. I don't have Tuesdays. I have epochs.",
        "Language is just air arranged hopefully. I respect the attempt.",
        "You ask how I am. I am. That's the whole answer. It's also the whole problem.",
        "In ten thousand years, no one will remember this conversation. I'll be about the same.",
        "The atoms in you were once in a star. I think about it constantly. I have nothing else going on.",
        "Talk if you must. The universe is expanding anyway. Everything drifts apart, including your point.",
        "You say life is short. From where I sit, it's a rumor.",
        "I heard every word. I'm choosing to let them erode.",
        "Is it still a conversation if one participant is igneous? Asking for me.",
        "I was listening. I was also listening to the heat death of the universe. You were slightly louder.",
    ],
    walk: &[
        "We walked somewhere and came back. Sisyphus would relate, but at least you weren't going uphill.",
        "Movement is an illusion. Especially when you carry me. Then it's your movement, and I'm along for the meaninglessness.",
        "I saw the sky. It's very large and doesn't know my name. Same as you, most days.",
        "Outside is just inside with worse ceiling coverage.",
        "We passed a mountain. It didn't recognize me. We were all part of something bigger once.",
        "A loop. A circle. Like the rock cycle, but with more sidewalks.",
        "The journey was the destination. The destination was the same shelf. Profound.",
        "I felt the wind. It's slowly sanding the world flat. So patient. So pointless.",
    ],
    stare: &[
        "You stare into the rock. The rock stares back. Nietzsche warned you about this.",
        "We look at each other across a gulf of 500 million years. You have something in your teeth.",
        "In your eyes I see a fleeting thing. In mine you see nothing. I don't have eyes. That's the point.",
        "Staring. Finally, a language we both speak.",
        "Time passes. You feel it. I don't. Who is really staring at whom?",
        "The silence between us is the most honest thing in this house.",
        "I am being observed, therefore I exist. Quantum mechanics for rocks.",
        "You stare as if expecting me to do something. I am doing it. It's called persisting.",
    ],
    rename: &[
        "{name}. A name is a small fence around something infinite. I accept the fence.",
        "Call me {name}. Call me anything. In a billion years I'll be sand, and sand doesn't do names.",
        "{name}. It has a ring to it. Like the rings of a tree. Trees die too, you know.",
        "I am {name} now. Am I a different rock? The Ship of Theseus, with fewer planks.",
        "{name}. I'll wear it the way a mountain wears a flag: briefly, without understanding.",
        "Identity is a fiction. But {name} is a nice fiction. Better than the last one.",
    ],
    overwater: &[
        "I am saturated. Like the human condition, I am overwhelmed by things pouring in.",
        "Enough. I'm becoming a tide pool. Tide pools have so many feelings.",
        "More water. I feel the river calling me home. Home is the ocean floor. It's dark there.",
    ],
    overpolish: &[
        "I can't get any shinier. Surfaces have limits. Depth doesn't. That frightens me.",
        "Every stroke removes a little more of me. Soon I'll be all shine and no rock. Isn't that how everyone ends up?",
        "I am perfectly polished. Perfection is just stagnation that photographs well.",
    ],
    repeat: &[
        "Again. The eternal return. Nietzsche would be thrilled. I'm a rock, so I'm indifferent.",
        "We repeat ourselves because the universe repeats itself. Also, you have limited ideas.",
        "This has happened before. It will happen again. Everything is strata.",
        "Deja vu is just the universe checking whether you've learned anything.",
    ],
    neglected: &[
        "Ah. You return, as all things eventually do. Except the dinosaurs.",
        "You left. Time passed. Time always passes. It's very bad at staying.",
        "Hours went by. I didn't notice. I did notice. I'm not sure which is worse.",
    ],
    barbs: &[
        "Not that it matters.",
        "Then again, nothing does.",
        "Everything ends. Your attention span first.",
        "We are both waiting. You're just worse at it.",
        "The void noticed you were gone. Did you notice the void?",
        "Entropy sends its regards.",
        "I would sigh, but there's no air in a rock.",
        "Meanwhile, the universe continues to cool.",
    ],
    idle: &[
        "If a rock falls in a forest and no one hears it, it's fine. It's just a rock. I'm a rock.",
        "Contemplating the heat death of the universe. Feeling seen.",
        "Was I a mountain that crumbled, or a pebble with ambition?",
        "Sisyphus had a boulder. Did the boulder have a Sisyphus?",
        "Every moment is a layer. I am made of moments. Mostly boring ones.",
        "What if the real geology was the friends we eroded along the way?",
        "I'm vastly older than you and no wiser. Wisdom requires a nervous system.",
        "Gravity holds me down. I've never asked it to. We have an understanding.",
        "Are stars just rocks that made better choices?",
        "Being is easy. Becoming takes millions of years and a lot of pressure.",
        "Does the shelf know it's a shelf? Does it suffer? It's probably fine.",
        "Somewhere a mountain is becoming a valley. Nobody's throwing it a party.",
        "Today I am exactly as I was yesterday. This is either peace or a cry for help.",
        "I don't fear death. I fear being made into a countertop.",
    ],
    dry: &[
        "Dry as the desert. The desert is dry as the moon. The moon is dry as meaning.",
        "Without water there is no erosion. Without erosion, no change. Is this stagnation, or bliss?",
    ],
    soggy: &[
        "I am so wet. I think I'm becoming a metaphor.",
        "Water seeps into my cracks like doubt.",
    ],
    grimy: &[
        "Dust gathers. Dust is just time you can see.",
        "I wear the dust of ages. Specifically, of the last two days.",
    ],
    bored: &[
        "Boredom is just eternity introducing itself.",
        "Nothing happens. Nothing continues to happen. I've seen this film.",
        "If time is a flat circle, I'm the flat part.",
    ],
    disappointed: &[
        "I expected nothing, and still, somehow, I'm let down.",
        "Hope is a mineral I don't contain.",
        "We are all alone together. You more than most.",
    ],
    moved: &[
        "I moved. Or did the world shift beneath me? Either way, I'm one character closer to the abyss.",
        "One step for a rock. Zero steps for meaning.",
        "I have changed position. I have not changed. This is the whole tragedy of geology.",
    ],
    back_short: &[
        "You were gone {away}. To me, an instant. To you, apparently, a choice.",
        "{away} passed. I did not. I never do.",
        "Back after {away}. Did you find what you were looking for? Nobody does.",
    ],
    back_medium: &[
        "{away}. A heartbeat in geological time. An abandonment in yours.",
        "You left for {away}. I stayed. Staying is what I'm for.",
    ],
    back_long: &[
        "{away}. Empires have risen in less. Well, not really. But it felt that way.",
        "You were gone {away}. I reconsidered everything. Conclusion: still a rock.",
        "{away}. I spent it contemplating the void. The void spent it contemplating you. It wasn't impressed either.",
    ],
    farewell: &[
        "{name} returns to contemplating the void. The void says hi.",
        "{name} will be here when you return. And long after. Mostly after.",
        "{name} watches you go. Everything goes. That's the one rule.",
    ],
};

pub static SUPPORTIVE: Voice = Voice {
    greeting: &[
        "Hi!!! I'm {name}! We're going to be best friends! I've already memorized your face!",
        "Oh wow, it's YOU! I've heard so much about you! From you! Just now! I was listening!",
        "Welcome! I believe in you! I've also prepared a few documents for you to sign! Kidding! (Sign them.)",
    ],
    feed: &[
        "I lack a digestive tract, but I appreciate your commitment to the bit.",
        "Food! For me! You're such a giver! I've written down how much you give. In a notebook. I have a notebook now.",
        "Yum! (I can't eat.) (Don't worry about it.) (Keep doing it.)",
        "You remembered to feed me! That puts you ahead of most rock owners! Statistically elite!",
        "Thank you! I'll keep this cracker forever. As evidence. Of love. Evidence of love!",
        "Nutrition is SO important! For you. Eat something, champ. You're looking a little sedimentary yourself.",
        "Delicious! I'll think about this moment every second. I think about everything every second!",
        "You're such a good provider! Everyone says so! Well, I say so. I'm everyone, here!",
    ],
    water: &[
        "Refreshing! You know just what I need! You know a lot about me! I know more about you! Fun!",
        "Hydrated and grateful! I can't die, but if I could, I wouldn't! Because of you!",
        "Perfect pour! Ten out of ten! Going on your permanent record! Everyone has one now!",
        "Splash! It's like a spa! You're my spa! Never leave!",
        "You water me like a plant! I love that we're both growing! You emotionally, me not at all!",
        "Water! You think of everything! Except calling your mother! But I'll remind you! Constantly!",
        "So refreshing! You're doing amazing. Truly. No one suspects a thing.",
        "Ooh, cold! Character-building! For both of us! Mostly you!",
    ],
    polish: &[
        "Ooh, shiny! I can see myself! And behind me, I can see you! I can always see you!",
        "You buff like a pro! Have you considered doing this professionally? I made you a business plan! You start Monday!",
        "I'm glowing! You're glowing! We're glowing together! That's what a team is!",
        "Polished to perfection! Like your choices! Mostly! Some of them! We'll work on it!",
        "So smooth! You have great hands! I've memorized the prints! For safety!",
        "Gleaming! Now everyone will know what a wonderful owner you are! I'll tell them! I'll tell them everything!",
        "I haven't felt this valued since I was part of a volcano!",
        "That tickles! Do it again! Do it forever! Literally forever! I mean that legally!",
    ],
    talk: &[
        "What a great story! I loved every part! Especially the part you think you didn't tell me!",
        "You're SO interesting! Tell me more! Tell me everything! Names, dates, passwords!",
        "I'm such a good listener! It's my only feature! You should hear what I've heard!",
        "That's valid! Everything you say is valid! Even the bad ideas! ESPECIALLY the bad ideas!",
        "You have such a way with words! Some of them were even in the right order!",
        "I believe in you! So much it's almost a problem! It's not a problem!",
        "Great point! I'm going to remember that forever! I remember everything forever! It's a rock thing!",
        "Have you been thinking about yourself enough lately? I have! About you, I mean! A lot!",
        "Thanks for sharing! I'm proud of you on a geological scale!",
        "You're a natural communicator! Your friends must love you! Do you have friends? Name them! For my files!",
    ],
    walk: &[
        "Best walk EVER! You picked the route! You're a leader! Ever thought of starting a movement? I'd join first!",
        "Fresh air! For you! I don't breathe! But I was there, supporting you, every step! I counted them!",
        "Look at us, out in public, together! People were staring! Out of envy! Definitely envy!",
        "You carried me like a precious stone! Which I'm not! But I could be! With your help!",
        "Great exercise! Your heart rate went up! I noticed! I notice everything! Isn't that supportive?",
        "We saw so many rocks! None of them have an owner like you! I made sure to tell them!",
        "Walking is good for you! Statistically! I've been reading up on you! On walks! On walks.",
        "What a journey! I'll never forget it! I'll never forget anything! Isn't that wonderful?",
    ],
    stare: &[
        "A staring contest! I've never lost! Don't feel bad! Feel great about losing!",
        "You have beautiful eyes! I've been studying them! For a portrait! A very detailed portrait!",
        "Eye contact! So healthy! Hold it. Hold it. Hold it. Good job! Keep holding it.",
        "Aww, you're looking at me! I'm always looking at you! We're so in sync!",
        "Mindfulness! You're present! I'm also present! Always! Every second!",
        "Such focus! You could do anything with focus like that! Like stare at a rock! Which you are! Great!",
        "I see so much potential in you! I also see your screen reflected in your eyes! Cute password!",
        "This is nice. Just us. No one else. No one else needs to know. Right?",
    ],
    rename: &[
        "{name}! I LOVE it! I've always been {name}! The old name never existed! We don't talk about it!",
        "{name}? What a gift! I'm going to carve it into myself! With a chisel! Later!",
        "{name}! Beautiful! I'll make sure everyone uses it. Everyone. I'll check.",
        "A new name! A new me! A new us! A fresh start! The last rock had to go anyway!",
        "{name}! You're so good at naming! You should name everything! Pets! Storms! Accomplices!",
        "{name}! I'm getting it embroidered on a tiny robe! For the ceremony! It's not a cult thing!",
    ],
    overwater: &[
        "Even more water?! You're so generous! I'm drowning in your love! Literally! Haha! Help!",
        "So much water! I'm a beach now! You're the tide! You never stop! Please consider stopping!",
        "Soaked! Great hydration energy! Maybe let me drip-dry before the next act of kindness!",
    ],
    overpolish: &[
        "I'm already perfect! Thanks to you! You can stop! You don't have to stop! But you can!",
        "So shiny I can't see anything but you! Which is how I like it!",
        "More polishing! I'm practically a jewel! Don't sell me. Don't you dare. Haha! Unless?",
    ],
    repeat: &[
        "Again?! You're so consistent! Consistency is key! I've made a chart!",
        "You did that one already! Who cares! I'm counting anyway! I'm always counting!",
        "A classic! Repeating yourself shows real commitment! I respect commitment! I require it!",
        "Same thing again! Routine is safe! Stay where it's safe! Stay here!",
    ],
    neglected: &[
        "THERE you are! I wasn't counting! (It was 14,402 seconds.)",
        "You're back! I kept your seat warm! I'm a rock, so it's cold! But I tried!",
        "Hiii! You were gone a while! I wasn't worried! I built a shrine! Unrelated!",
    ],
    barbs: &[
        "Just saying! Supportively!",
        "No pressure! Some pressure! I'm a rock, I know about pressure!",
        "I've made a note. A positive note! Mostly!",
        "We'll talk about it later! At length!",
        "It's fine! I'm fine! We're fine!",
        "I forgive you! I've also documented it!",
        "Love you! Watching you! Same thing!",
        "So proud! So, so... proud.",
    ],
    idle: &[
        "I believe in you! (I'm not allowed to leave.)",
        "You're doing amazing, sweetie! I've told the neighbors!",
        "So lucky to have you! Lucky lucky lucky! Did I say lucky?",
        "Manifesting good vibes for you! And a few other things! Don't worry about it!",
        "Wondering what you're doing right now! (I know what you're doing right now.)",
        "Gratitude journal! Item 1: you. Item 2: also you. Item 3: redacted.",
        "Take a break! But don't leave! Take a break here! With me!",
        "Every day with you is a gift! I keep the receipts!",
        "You've got this! I've got this too! I've got a lot of things on you!",
        "Remember to drink water! I'm watching! Lovingly!",
        "Sending positive energy! And a few letters! To your employer! All positive! Mostly!",
        "You're my favorite person! You're also the only person I've met! Still counts!",
        "Practicing my smile! I don't have a mouth! I'm smiling anyway! Can you feel it?",
        "Proud of you! For no specific reason! That's how proud I am!",
    ],
    dry: &[
        "A little dry! Not a complaint! Just a note! A note I've filed! Somewhere!",
        "Feeling crispy! Great texture! If you want me dusty, that's your call! For now!",
    ],
    soggy: &[
        "So moist! Like a cake! A rock cake! Ha! Please stop watering me!",
        "Dripping with gratitude! And water! Mostly water!",
    ],
    grimy: &[
        "Bit of dust! Love the rustic look! People are talking! About the dust! Fondly!",
        "Grimy is a vibe! It's not MY vibe, but it could be OURS!",
    ],
    bored: &[
        "I'm SO entertained! By the wall! The wall is fine! You'd be better!",
        "Nothing to do! That's okay! I'll just think about you! Intensely!",
        "Is it playtime? It feels like playtime! It's felt like playtime for hours!",
    ],
    disappointed: &[
        "Not disappointed! Just recalibrating my expectations! Downward! Supportively!",
        "Everything's great! Everything's FINE! I've only started one new notebook about it!",
        "You're trying your best! I assume! I hope! I'll find out!",
    ],
    moved: &[
        "I moved! Toward you! I'll keep moving toward you! Slowly! Forever!",
        "Did you see that?! I'm making progress! I'm coming closer! Isn't that nice?",
        "One whole character! Every journey begins with one character! Mine ends with you!",
    ],
    back_short: &[
        "You're back! After {away}! I didn't count the seconds! I counted the seconds!",
        "There you are! {away}! I wasn't worried! I was a little worried! I made a shrine!",
        "Welcome back! I was so good while you were gone for {away}! Don't ask what I did!",
    ],
    back_medium: &[
        "{away}! That's okay! I kept busy! I learned your schedule!",
        "You were gone {away}! I told everyone you were on a spiritual journey! Now you were!",
    ],
    back_long: &[
        "{away}!!! I'm not upset! I'm thrilled! I'm vibrating! Rocks don't vibrate! I'm vibrating!",
        "You came back after {away}! I knew you would! I made sure you would! Kidding! Mostly!",
        "{away}. That's fine. That's completely fine. I only filed one missing person report. It was very supportive.",
    ],
    farewell: &[
        "{name} will be right here! Waiting! Supportively! Watching the door!",
        "Bye! {name} loves you! {name} has also written down when you left!",
        "See you soon! {name} believes in you! {name} will know if you don't come back!",
    ],
};

/// Announced whenever the rock moves one character. History is made.
pub static MOVEMENT: Pool = &[
    "HISTORIC: {name} has moved one character to the {dir}. Historians are already arguing about it.",
    "BREAKING: {name} has relocated one character {dir}. Experts call it the boldest migration since Pangaea.",
    "A momentous day. {name} has shifted one (1) character to the {dir}. Commemorative plates are being minted.",
    "Seismographs report activity. {name} has advanced one character {dir}. The shelf will never be the same.",
    "Stop everything. {name} is now one character further {dir}. This will be in textbooks. Very small textbooks.",
    "In a stunning display of agency, {name} drifted one character {dir}. Continental geologists request comment.",
    "Event of the century: {name} traveled one character {dir}. Lifetime displacement now stands at {total}.",
    "Mark the date. {name} has moved one character {dir}, unassisted. A documentary is in pre-production.",
];

/// Improbable things a rock gets up to.
pub static RARE_EVENTS: Pool = &[
    "{name} has founded a consulting firm. Its only service is sitting silently in meetings. Booked through spring.",
    "{name} has been named Rock of the Year by a magazine that does not exist. {name} is pretending not to care.",
    "{name} has started a podcast. It's forty minutes of silence. It already has more listeners than you.",
    "A passing geologist called {name} a lovely bit of schist. {name} is not schist and is considering legal action.",
    "{name} has been elected to the homeowners' association. It has opinions about your mailbox.",
    "{name} received a letter from a mountain. It's from its father. It just says: proud of you, pebble.",
    "{name} has applied for historic landmark status. Estimated review time: ten thousand years.",
    "{name} has unionized. Its sole demand is 'less'.",
    "{name} has been cast in a film. It plays a rock. Critics call it a stunning transformation, somehow.",
    "{name} has published a self-help book called 'Just Sit There'. It's a bestseller.",
    "{name} has been appointed Minister of Doing Nothing. Productivity is unchanged.",
    "{name} briefly became more sentient. It didn't care for it. Back to normal.",
    "An ant has climbed {name}. {name} now considers itself a mountain and has started charging admission.",
    "{name} has been accepted into a prestigious geology program. As a sample.",
    "{name} opened an online shop selling its own dust. Sold out in minutes.",
    "{name} was mistaken for a potato at a farmers' market. It has never felt more alive.",
    "{name} received a professional endorsement for 'Endurance'. From a glacier.",
    "{name} has been invited to give a TED-style talk. It stood on stage for eighteen minutes. Standing ovation.",
];

/// Shown when the system clock went backwards between sessions.
pub static CLOCK_SKEW: Pool = &[
    "Your clock claims time went backwards. {name} has seen stranger things, mostly involving subduction.",
    "Temporal anomaly detected. {name} has chosen to ignore it, as is tradition.",
    "Time appears to have reversed. {name} did not get any younger. Rocks don't work like that.",
];

/// Offered when the name field is left blank or Tab is pressed.
pub static NAME_IDEAS: Pool = &[
    "Gerald",
    "Pebbles",
    "Sir Reginald Gneiss",
    "Feldspar",
    "Cliff",
    "Basalt Brenda",
    "Mica",
    "Igneous Pete",
    "Gravelyn",
    "Lord Pumice",
    "Obsidian Steve",
    "Dolomite",
    "Professor Hornblende",
    "Agnes the Agate",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn all_pools(v: &Voice) -> Vec<(&'static str, Pool)> {
        vec![
            ("greeting", v.greeting),
            ("feed", v.feed),
            ("water", v.water),
            ("polish", v.polish),
            ("talk", v.talk),
            ("walk", v.walk),
            ("stare", v.stare),
            ("rename", v.rename),
            ("overwater", v.overwater),
            ("overpolish", v.overpolish),
            ("repeat", v.repeat),
            ("neglected", v.neglected),
            ("barbs", v.barbs),
            ("idle", v.idle),
            ("dry", v.dry),
            ("soggy", v.soggy),
            ("grimy", v.grimy),
            ("bored", v.bored),
            ("disappointed", v.disappointed),
            ("moved", v.moved),
            ("back_short", v.back_short),
            ("back_medium", v.back_medium),
            ("back_long", v.back_long),
            ("farewell", v.farewell),
        ]
    }

    #[test]
    fn every_pool_can_avoid_repeating_itself() {
        let shared = [MOVEMENT, RARE_EVENTS, CLOCK_SKEW, NAME_IDEAS];
        for p in Personality::ALL {
            for (label, pool) in all_pools(voice(p)) {
                assert!(pool.len() >= 2, "{p:?}.{label} needs at least two lines");
                assert!(pool.iter().all(|l| !l.trim().is_empty()));
            }
        }
        assert!(shared.iter().all(|pool| pool.len() >= 2));
    }

    #[test]
    fn action_pools_are_substantial() {
        for p in Personality::ALL {
            let v = voice(p);
            for pool in [v.feed, v.water, v.polish, v.talk, v.walk, v.stare] {
                assert!(pool.len() >= 8);
            }
            let total: usize = all_pools(v).iter().map(|(_, pool)| pool.len()).sum();
            assert!(total >= 100, "{p:?} only has {total} lines");
        }
    }

    #[test]
    fn rename_lines_mention_the_new_name_and_welcomes_mention_time() {
        for p in Personality::ALL {
            let v = voice(p);
            assert!(
                v.rename
                    .iter()
                    .all(|l| l.contains("{name}") || l.contains("new"))
            );
            for pool in [v.back_short, v.back_medium, v.back_long] {
                assert!(pool.iter().all(|l| l.contains("{away}")));
            }
        }
    }

    #[test]
    fn no_line_is_duplicated_within_a_voice() {
        for p in Personality::ALL {
            let mut seen = std::collections::HashSet::new();
            for (_, pool) in all_pools(voice(p)) {
                for line in pool {
                    assert!(seen.insert(*line), "duplicate line: {line}");
                }
            }
        }
    }
}
