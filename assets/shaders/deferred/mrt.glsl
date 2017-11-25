Vertex <
    #version 450 core
    
    #extension GL_ARB_separate_shader_objects : enable
    #extension GL_ARB_shading_language_420pack : enable
    
    layout (location = 0) in vec3 inPosition;
    layout (location = 1) in vec3 inTangent;
    layout (location = 2) in vec3 inNormal;
    layout (location = 3) in vec2 inUv;
    
    layout (location = 0) out vec3 outWorldPos;
    layout (location = 1) out vec3 outNormal;
    layout (location = 2) out vec3 outTangent;
    layout (location = 3) out vec2 o_uv;
    
    layout (binding = 0) uniform UBO
    {
        mat4 projection;
        mat4 view;
    } ubo;
    
    layout (binding = 3) uniform Model
     {
        mat4 m;
     } model;
    
    void main() {
        vec4 WorldPos = model.m * vec4(inPosition, 1.0);
        outWorldPos = WorldPos.xyz;
        gl_Position = ubo.projection * ubo.view * WorldPos;
    
        o_uv = inUv;
        o_uv.t = 1.0 - o_uv.t;
    
        // GL to Vulkan coord space
        outWorldPos.y = -outWorldPos.y;
    
        // Normal in world space
        mat3 mNormal = transpose(inverse(mat3(model.m)));
        outNormal = mNormal * normalize(inNormal);
        outTangent = mNormal * normalize(inTangent);
    }
>

Fragment <
    #version 450 core
    
    #extension GL_ARB_separate_shader_objects : enable
    #extension GL_ARB_shading_language_420pack : enable
    
    layout (binding = 1) uniform sampler2D dTexture;
    layout (binding = 2) uniform sampler2D noramlmap;
    
    layout (location = 0) in vec3 outWorldPos;
    layout (location = 1) in vec3 outNormal;
    layout (location = 2) in vec3 outTangent;
    layout (location = 3) in vec2 o_uv;
    
    layout (location = 0) out vec4 gPosition;
    layout (location = 1) out vec4 gNormal;
    layout (location = 2) out vec4 gcolor;
    
    void main(){
        gPosition   = vec4(outWorldPos,1.0);
    // Calculate normal in tangent space
        vec3 N = normalize(outNormal);
        N.y = -N.y;
        vec3 T = normalize(outTangent);
        vec3 B = cross(N, T);
        mat3 TBN = mat3(T, B, N);
        vec3 tnorm = TBN * normalize(texture(noramlmap, o_uv).xyz * 2.0 - vec3(1.0));
        gNormal     = vec4(N, 1.0);
        gcolor      = texture(dTexture, o_uv);
    }
>