// 10 - Statistics of numbers typed in
//
// Reads one number per line until an empty line (or the end of the input),
// skips anything that is not a number, then prints a summary.
//
//   tiny-compiler-vm execute 10_statistics.tb < 10_statistics.in

float sum = 0.0;
float count = 0.0;                     // a float, so that sum / count needs no conversion
float? smallest = null;
float? largest = null;
int skipped = 0;
bool reading = true;

print "Enter numbers, one per line. Finish with an empty line.";

while (reading) {
    string? line = input();

    if (line == null) {
        reading = false;               // end of input
    } else if (line!! == "") {
        reading = false;               // empty line
    } else {
        float? parsed = parseFloat(line);

        if (parsed == null) {
            skipped = skipped + 1;
        } else {
            float value = parsed!!;
            sum = sum + value;
            count = count + 1.0;

            // || stops at the first true side, so smallest!! is only read once it is set
            if (smallest == null || value < smallest!!) { smallest = value; }
            if (largest == null || value > largest!!) { largest = value; }
        }
    }
}

if (count == 0.0) {
    print "No numbers were entered.";
} else {
    print "count:";
    print count;
    print "sum:";
    print sum;
    print "average:";
    print sum / count;
    print "smallest:";
    print smallest!!;
    print "largest:";
    print largest!!;
}

print "skipped:";
print skipped;
