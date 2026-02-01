# 🧪 Testing Guide - Word Wolf Game

## 🚀 How to Run the Server

```bash
# Start the server on default port (8080)
nix develop --command cargo run

# Or on a custom port
nix develop --command cargo run 0.0.0.0:3000
```

You should see:
```
🐺 Word Wolf Server Starting...

✓ Server listening on 127.0.0.1:8080
✓ Room manager initialized

📋 Available endpoints:
  GET  /                    - Serve login.html
  GET  /events?room_id=X    - SSE connection for room X
  POST /room/create         - Create a new room
  POST /room/join           - Join a room
  POST /room/ready          - Mark player as ready
  POST /room/vote           - Submit a vote
  GET  /room/list           - List all rooms
  GET  /room/state?room_id=X - Get room state

🎮 Server ready for connections!
```

---

## 🌐 Testing the Frontend

### **Test Flow 1: Create a Room (Single Browser)**

1. **Open browser:** `http://localhost:8080`
2. **Login page:** Enter a name (e.g., "Alice") → Click 決定
3. **Home page:** You should see "ようこそ、Aliceさん！"
4. **Create room:** Click "部屋を作る"
5. **Room creation:**
   - 部屋名: "Test Room"
   - 参加人数: 4
   - 狼の数: 1
   - ルームID: "room123"
   - ジャンル: 食べ物
   - Click "作成"
6. **Waiting room:** You should be in stay.html
   - Shows "ルームID: room123"
   - Click "準備完了" button
   - You'll see SSE messages in the status

### **Test Flow 2: Join a Room (Second Browser/Incognito)**

1. **Open incognito/another browser:** `http://localhost:8080`
2. **Login:** Enter different name (e.g., "Bob")
3. **Home page:** Click "部屋に参加する"
4. **Join room:** Enter "room123" → Click "参加"
5. **Waiting room:** Both browsers should receive SSE updates!

---

## 🔍 What to Check

### **Browser Console (F12)**

Open Developer Tools and check Console for:
```
Logged in as: Alice ID: player_1234567890_abc123
```

### **Server Logs**

Watch the terminal where cargo run is running:
```
GET /
POST /room/create
POST /room/join
GET /events room_id=room123
POST /room/ready
```

### **Network Tab (F12 → Network)**

You should see:
- **POST requests** to /room/create, /room/join, /room/ready
- **EventStream** connection to /events (stays open for SSE)

---

## 📱 Testing SSE (Real-time Updates)

1. **Open 2 browsers side by side**
2. **Browser 1:** Create room "test123" as "Alice"
3. **Browser 2:** Join room "test123" as "Bob"
4. **Browser 1:** Click "準備完了"
5. **Browser 2:** Should see message "Player joined" or similar in status
6. **Browser 2:** Click "準備完了"
7. **Both browsers:** Should see "All players ready! Starting game..." and navigate to theme.html

---

## 🐛 Common Issues

### **Issue: "Connection error" in stay.html**

**Cause:** SSE connection failed
**Fix:** Check server is running and room exists

**Debug:**
```bash
# Check if server is running
curl http://localhost:8080/

# Check if room exists
curl "http://localhost:8080/room/state?room_id=room123"
```

### **Issue: Buttons do nothing**

**Cause:** JavaScript not loaded or browser cache
**Fix:** Hard refresh (Ctrl+Shift+R or Cmd+Shift+R)

### **Issue: "Failed to parse request"**

**Cause:** Malformed request body
**Fix:** Check browser console for fetch errors

---

## 🧪 API Testing with curl

### **Create a room:**
```bash
curl -X POST http://localhost:8080/room/create \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "room_id=test123&room_name=TestRoom&max_players=4&wolf_count=1&genre=Food"
```

### **Join a room:**
```bash
curl -X POST http://localhost:8080/room/join \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "room_id=test123&player_id=player1&player_name=Alice"
```

### **Mark ready:**
```bash
curl -X POST http://localhost:8080/room/ready \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "room_id=test123&player_id=player1"
```

### **List rooms:**
```bash
curl http://localhost:8080/room/list
```

### **Connect to SSE (keeps connection open):**
```bash
curl -N http://localhost:8080/events?room_id=test123
```

---

## ✅ Expected Behavior

### **When room is created:**
- Server log: `POST /room/create`
- Response: `{"room_id":"room123"}`
- Player automatically joins the room
- Navigates to stay.html

### **When player joins:**
- Server log: `POST /room/join`
- SSE broadcast: "Player [name] joined"
- All connected clients in that room receive the message

### **When player marks ready:**
- Server log: `POST /room/ready`
- SSE broadcast: "Player [name] is ready"
- Button changes to "準備完了！" (green, disabled)

### **When all players ready:**
- SSE broadcast: "All players ready! Starting game..."
- Game starts (roles & themes assigned)
- SSE broadcast: "Game started! Check your roles and themes."
- All clients navigate to theme.html after 1 second

---

## 📊 Data Flow Diagram

```
Browser 1                Server                  Browser 2
    │                      │                          │
    │──login.html──────────┼──────────────────────────│
    │  (Alice)             │                          │
    │                      │                          │
    │──POST /room/create──>│                          │
    │<─{"room_id":"123"}───│                          │
    │──POST /room/join────>│                          │
    │                      │                          │
    │                      │<────login.html───────────│
    │                      │        (Bob)             │
    │                      │                          │
    │                      │<────POST /room/join──────│
    │<══SSE: Player joined═│═══SSE: Player joined════>│
    │                      │                          │
    │──GET /events────────>│<────GET /events──────────│
    │  (SSE open)          │          (SSE open)      │
    │                      │                          │
    │──POST /room/ready───>│                          │
    │<══SSE: Alice ready═══│═══SSE: Alice ready══════>│
    │                      │                          │
    │                      │<────POST /room/ready─────│
    │<══SSE: Bob ready═════│═══SSE: Bob ready════════>│
    │                      │                          │
    │<══SSE: Game started═>│<══SSE: Game started═════>│
    │                      │                          │
    │──Navigate theme.html─┼──Navigate theme.html────│
```

---

## 🎯 Next Steps

After confirming the basic flow works:

1. ✅ **Theme display page** (theme.html) - Show player their assigned theme
2. ✅ **Game discussion page** (game.html) - Main discussion phase
3. ✅ **Voting mechanism** - Let players vote
4. ✅ **Results page** - Show who won

---

**Happy Testing!** 🎮
