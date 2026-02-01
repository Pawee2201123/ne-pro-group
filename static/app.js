let myConnId = null; // 接続ID（唯一のID）

const ws = new WebSocket("ws://localhost:3000/ws");
const chatSse = new EventSource("/chat/events");

ws.onopen = () => console.log("WS OPEN");

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log("WS MSG", msg);
  if (msg.type === "welcome") {
    myConnId = msg.conn_id;
    updateMyId(false);
    return;
  }

  if (msg.type === "your_topic") {
    document.getElementById("topic").textContent = msg.topic;
    return;
  }

  // ★ それ以外は state
  renderFromState(msg);
};

ws.onerror = (e) => {
  console.error("WS ERROR", e);
};

ws.onclose = () => {
  console.log("WS CLOSED");
};


function join() {
  ws.send(JSON.stringify({ type: "join" }));
}

function startGame() {
  ws.send(JSON.stringify({ type: "start" }));
}


function vote(id) {
  ws.send(JSON.stringify({ type: "vote", target: Number(id) }));
}

function resetGame() {
  ws.send(JSON.stringify({ type: "reset" }));
}

// joined: trueならID表示、falseなら未参加表示
function updateMyId(joined) {
  const text = joined ? String(myConnId ?? "-") : "未参加";
  document.querySelectorAll(".myId").forEach(el => el.textContent = text);
}

function show(id) {
  document.querySelectorAll(".screen")
    .forEach(s => s.classList.remove("active"));
  document.getElementById(id).classList.add("active");
}

function submitKeyword() {
  const input = document.getElementById("keywordInput");
  const kw = input.value.trim();
  if (!kw) return;
  fetch(`/submit_keyword?player_id=${myConnId}`, {
    method: "POST",
    headers: {"Content-Type": "application/json"},
    body: JSON.stringify({keyword: kw}),
  });
  input.value = "";
}

chatSse.onmessage = (e) => {
  if (e.data === "__CLEAR__") {
    document.getElementById("chatBox").innerHTML = "";
    return;
  }

  const box = document.getElementById("chatBox");
  box.innerHTML += `<div>${e.data}</div>`;
  box.scrollTop = box.scrollHeight;
};

function sendSpeak() {
  const input = document.getElementById("chatInput");
  const msg = input.value.trim();
  if (!msg) return;

  // WSで発言回数減少
  ws.send(JSON.stringify({ type: "speak" }));

  // POSTでチャット送信
  fetch("/chat/send", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ msg: `Player${myConnId}: ${msg}` }),
  });

  input.value = ""; // 入力欄クリア
}

function renderFromState(state) {
  if (!state || !state.phase) return;

  switch (state.phase) {
    case "Waiting":
      renderWaiting(state);
      break;

    case "ThemeSelection":
      renderThemeSelection(state);
      break;

    case "Playing":
      renderPlaying(state);
      break;

    case "Voting":
      renderVoting(state);
      break;

    case "Result":
      renderResult(state);
      break;
  }
}

function renderWaiting(state) {
  show("waiting");

  document.getElementById("playerCount").textContent =
    Object.keys(state.players).length;

  const joined = myConnId && state.players[myConnId];

  document.getElementById("joinBtn").disabled = !!joined;
  document.getElementById("startBtn").disabled =
    Object.keys(state.players).length < 2;

  updateMyId(!!joined);
}

function renderThemeSelection(state) {
  show("themeSelection");

  document.getElementById("theme").textContent =
    state.genre ?? "未設定";

  updateMyId(!!(myConnId && state.players[myConnId]));
}

function renderPlaying(state) {
  show("playing");

  const joined = myConnId && state.players[myConnId];
  const me = joined ? state.players[myConnId] : null;

  document.getElementById("gameTimer").textContent =
    state.remaining_time;

  document.getElementById("remainingSpeak").textContent =
    me ? me.remaining_speak : "-";

  const speakBtn = document.getElementById("speakBtn");
  speakBtn.disabled = !me || me.remaining_speak === 0;

  updateMyId(!!joined);
}

function renderVoting(state) {
  show("voting");

  const list = document.getElementById("voteButtons");
  list.innerHTML = "";

  const me = myConnId && state.players[myConnId];
  const myVote = me ? me.vote : null;

  Object.values(state.players).forEach(p => {
    const btn = document.createElement("button");
    btn.textContent = `Player ${p.id}`;

    if (myVote === p.id) {
      btn.style.backgroundColor = "yellow";
    } else {
      btn.style.backgroundColor = "";
    }

    btn.onclick = () => vote(p.id);
    list.appendChild(btn);
  });

  document.getElementById("voteTimer").textContent =
    state.voting_time;

  updateMyId(!!me);
}

function renderResult(state) {
  show("result");

  const winner = document.getElementById("winner");
  winner.textContent =
    state.is_villager_win ? "村人の勝利！" : "人狼の勝利！";

  document.getElementById("executed").textContent =
    state.executed_id
      ? `Player ${state.executed_id}`
      : "処刑なし";

  document.getElementById("wolf").textContent =
    state.wolf_id
      ? `Player ${state.wolf_id}`
      : "-";

  updateMyId(true);
}