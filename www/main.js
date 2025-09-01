import init, { GameEngine } from './pkg/jubilee_engine.js';

async function run() {
    await init();
    const boardJson = await fetch('./board.json').then(r => r.text());
    const actionScript = await fetch('./scripts/action.rhai').then(r => r.text());
    const cycleScript = await fetch('./scripts/cycle.rhai').then(r => r.text());

    // --- 엔진 생성 시 플레이어 수 전달 ---
    const playerCount = 2; // 우선 2명으로 시작
    const engine = new GameEngine(boardJson, playerCount);

    const rollDiceBtn = document.getElementById('roll-dice-btn');
    const endTurnBtn = document.getElementById('end-turn-btn'); // 새로 추가될 버튼
    const turnDisplay = document.getElementById('turn-display'); // 현재 턴 표시
    const stateDisplay = document.getElementById('state-display');
    const logDisplay = document.getElementById('log');

    function render() {
        const state = JSON.parse(engine.get_state_as_json());
        stateDisplay.textContent = JSON.stringify(state, null, 2);
        logDisplay.innerHTML = state.log.join('<br>');
        logDisplay.scrollTop = logDisplay.scrollHeight;

        // 현재 턴 플레이어 정보 표시
        const currentPlayer = state.players[state.current_turn_idx];
        turnDisplay.textContent = `Current Turn: Player ${currentPlayer.id}`;
    }

    rollDiceBtn.addEventListener('click', () => {
        const diceRoll = Math.floor(Math.random() * 6) + 1;
        try {
            engine.run_turn_script(actionScript, BigInt(diceRoll));
            render();
            rollDiceBtn.disabled = true; // 한 턴에 한 번만 굴리도록 비활성화
            endTurnBtn.disabled = false;
        } catch (e) {
            console.error("Error executing turn script:", e);
            logDisplay.innerHTML += `<br><span style="color:red;">ERROR: ${e}</span>`;
        }
    });

    endTurnBtn.addEventListener('click', () => {
        engine.end_turn(); // Rust의 end_turn 함수 호출
        render();
        rollDiceBtn.disabled = false; // 다음 플레이어를 위해 버튼 다시 활성화
        endTurnBtn.disabled = true;
    });

    render();
}
run();