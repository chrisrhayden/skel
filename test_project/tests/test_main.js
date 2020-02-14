const { run } = require('./main');

function test_main() {
    if (!run()) {
        console.log("its fucked");
    }
}

test_main();
