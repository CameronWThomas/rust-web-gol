int updateFrameRate =1;
vec4 onCol = vec4(0.0,1.0,0.0,1.0);
vec4 offCol = vec4(0.0,0.0, 0.0, 1.0);

float rand(vec2 co) {
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

vec2 wrapCoord(vec2 coord) {
    if (coord.x < 0.0) coord.x = iResolution.x;
    if (coord.y < 0.0) coord.y = iResolution.y;
    return coord;
}
bool isAlive (in vec2 fragCoord){
    vec4 myCol = texture(iChannel0, fragCoord / iResolution.xy);
    bool curAlive = myCol == onCol;
    int aliveCount = 0;
    
    for (int x=-1; x<2; x+=1){
        for(int y = -1; y <2; y++){
            if(x == 0 && y == 0){
                continue;
            }
            else{
                vec2 samplePos = wrapCoord(fragCoord + vec2(x,y));
                vec4 col = texture(iChannel0, samplePos / iResolution.xy);
                if(col == onCol) aliveCount++;
            }
        }
    }
    if(!curAlive){
        if(aliveCount == 3) return true;
        return false;
    }
    if(aliveCount < 2) return false;
    if(aliveCount > 1 && aliveCount < 4) return true;
    if(aliveCount > 3) return false;
    
    
}
void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    if (iFrame < 2) {
        float onOff = step(0.5, rand(fragCoord));
        if(onOff > 0.0) fragColor = onCol;
        else fragColor = offCol;
    } else {
        bool update = iFrame % updateFrameRate == 0;
        if(!update){
            fragColor = texture(iChannel0, fragCoord / iResolution.xy);
            return;
        }
       
        // MoveDiagonally
        //vec2 diagCoord = wrapCoord(fragCoord - vec2(1.0,1.0));
        //fragColor = texture(iChannel0, diagCoord / iResolution.xy);
        bool alive = isAlive(fragCoord);
        if(alive) fragColor = onCol;
        else fragColor = offCol;
    }
}

