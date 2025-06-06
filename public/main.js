console.log("hello world");

let header = document.querySelector("h1");
let start_index = 0;
let left = false;

function animate() {

    if (start_index >= 360) {
        left = true;
    }
    if (start_index <= 0) {
        left = false;
    }

    if (left) {
        start_index -= 1;
    } else {
        start_index += 1;
    }
    header.style.transform = `translate(${start_index}px)`;

    requestAnimationFrame(animate);
}

animate();

